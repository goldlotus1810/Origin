// internal/perception/audio.go
// AudioSkill — phân tích âm thanh → Spline formant → ISL address
// Không lưu raw waveform. Mọi âm thanh = Spline harmonic.

package perception

import (
	"math"

	"github.com/goldlotus1810/HomeOS/internal/gene"
	"github.com/goldlotus1810/HomeOS/internal/isl"
)

// FormantSpline biểu diễn âm thanh qua 3 formant chính F1,F2,F3
// Mỗi formant = Spline theo thời gian
type FormantSpline struct {
	F1, F2, F3 *gene.Spline // Hz formant trajectories
	Duration   float64      // giây
	Amplitude  float64
}

// AudioFrame là một khung âm thanh ngắn
type AudioFrame struct {
	Samples    []float64 // raw PCM samples
	SampleRate int       // Hz
}

// AudioSkill phân tích AudioFrame → FormantSpline → ISL
type AudioSkill struct {
	SampleRate int
}

func NewAudioSkill(sampleRate int) *AudioSkill {
	return &AudioSkill{SampleRate: sampleRate}
}

// Analyze chuyển frame âm thanh thành FormantSpline
func (a *AudioSkill) Analyze(frame AudioFrame) *FormantSpline {
	if len(frame.Samples) == 0 {
		return nil
	}
	// DFT đơn giản để tìm frequencies nổi bật
	freqs := a.dft(frame.Samples, frame.SampleRate)

	// Tìm 3 formants (F1 thấp, F2 giữa, F3 cao)
	f1 := findPeak(freqs, 200, 900)
	f2 := findPeak(freqs, 900, 2500)
	f3 := findPeak(freqs, 2500, 3500)

	dur := float64(len(frame.Samples)) / float64(frame.SampleRate)

	// Mỗi formant → Spline 4 keyframe (đơn giản hóa)
	return &FormantSpline{
		F1:       freqToSpline(f1),
		F2:       freqToSpline(f2),
		F3:       freqToSpline(f3),
		Duration: dur,
		Amplitude: rms(frame.Samples),
	}
}

// ToISL ánh xạ FormantSpline → ISL address
// Dựa trên tỉ lệ F1/F2 → nhận biết vowel → ISL[K][Script][?][char]
func (a *AudioSkill) ToISL(fs *FormantSpline) isl.Address {
	if fs == nil {
		return isl.Address{}
	}
	// Eval formants tại midpoint
	f1 := fs.F1.Eval(0.5)
	f2 := fs.F2.Eval(0.5)

	// Phân loại vowel theo F1/F2 space (Peterson & Barney 1952)
	// F1 thấp + F2 cao → /i/ (front high)
	// F1 cao + F2 thấp → /a/ (open)
	// F1 thấp + F2 thấp → /u/ (back high)
	ratio := f1.X / (f2.X + 1)

	var id byte
	switch {
	case ratio < 0.15: // /i/
		id = 'i'
	case ratio < 0.25: // /e/
		id = 'e'
	case ratio < 0.40: // /a/
		id = 'a'
	case ratio < 0.55: // /o/
		id = 'o'
	default: // /u/
		id = 'u'
	}

	return isl.Address{Layer: 'K', Group: 'S', Type: 'v', ID: id}
}

// dft tính DFT đơn giản, trả về magnitude per frequency bin
func (a *AudioSkill) dft(samples []float64, sr int) map[float64]float64 {
	n := len(samples)
	if n == 0 {
		return nil
	}
	result := make(map[float64]float64, n/2)

	// DFT O(n²) — chỉ tính các bin cần thiết (200-4000 Hz)
	for k := 1; k < n/2; k++ {
		freq := float64(k*sr) / float64(n)
		if freq < 100 || freq > 4000 {
			continue
		}
		re, im := 0.0, 0.0
		for t, s := range samples {
			angle := 2 * math.Pi * float64(k) * float64(t) / float64(n)
			re += s * math.Cos(angle)
			im -= s * math.Sin(angle)
		}
		result[freq] = math.Sqrt(re*re+im*im) / float64(n)
	}
	return result
}

// findPeak tìm frequency có magnitude cao nhất trong [fLow, fHigh]
func findPeak(freqs map[float64]float64, fLow, fHigh float64) float64 {
	best, bestMag := fLow, 0.0
	for f, mag := range freqs {
		if f >= fLow && f <= fHigh && mag > bestMag {
			bestMag = mag
			best = f
		}
	}
	return best
}

// freqToSpline tạo Spline đơn từ một frequency Hz
func freqToSpline(freqHz float64) *gene.Spline {
	// Normalize Hz → [0..1] space (100-4000 Hz range)
	norm := (freqHz - 100) / 3900
	return &gene.Spline{Keys: []gene.Keyframe{
		{T: 0.0, Vec3: gene.Vec3{X: norm, Y: freqHz / 1000.0}},
		{T: 0.33, Vec3: gene.Vec3{X: norm * 1.05, Y: freqHz / 1000.0 * 1.02}},
		{T: 0.66, Vec3: gene.Vec3{X: norm * 0.98, Y: freqHz / 1000.0 * 0.99}},
		{T: 1.0, Vec3: gene.Vec3{X: norm, Y: freqHz / 1000.0}},
	}}
}

// rms tính Root Mean Square amplitude
func rms(samples []float64) float64 {
	if len(samples) == 0 {
		return 0
	}
	sum := 0.0
	for _, s := range samples {
		sum += s * s
	}
	return math.Sqrt(sum / float64(len(samples)))
}

// CanHandle — Skill interface
func (a *AudioSkill) CanHandle(msg *isl.ISLMessage) bool {
	return msg.MsgType == isl.MsgQuery && msg.PrimaryAddr.Group == 'A' &&
		msg.PrimaryAddr.Type == 'u' // audio
}

// Name — Skill interface
func (a *AudioSkill) Name() string { return "audio" }
