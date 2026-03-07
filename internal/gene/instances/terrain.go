// internal/gene/instances/terrain.go
// TerrainGene — FBM fractal terrain
// DNA: ∫(fbm, oct:6, seed:888)

package instances

import (
	"fmt"

	"github.com/goldlotus1810/HomeOS/internal/gene"
)

// TerrainGene biểu diễn terrain FBM
type TerrainGene struct {
	OriginX, OriginZ float64
	Scale            float64 // world units per terrain unit
	HeightScale      float64
	Octaves          int
	Seed             float64
	WaterLevel       float64
}

func NewTerrainGene(ox, oz, scale, heightScale float64, oct int, seed float64) *TerrainGene {
	return &TerrainGene{
		OriginX: ox, OriginZ: oz,
		Scale: scale, HeightScale: heightScale,
		Octaves: oct, Seed: seed,
		WaterLevel: 0.3,
	}
}

// Height trả về chiều cao terrain tại (x,z) world space
func (t *TerrainGene) Height(x, z float64) float64 {
	lx := (x - t.OriginX) / t.Scale
	lz := (z - t.OriginZ) / t.Scale
	h := gene.FBM(lx, lz, t.Octaves, t.Seed)
	return h * t.HeightScale
}

func (t *TerrainGene) SDF(p gene.Vec3) float64 {
	h := t.Height(p.X, p.Z)
	return p.Y - h // trên mặt đất = dương, dưới = âm
}

func (t *TerrainGene) Normal(p gene.Vec3) gene.Vec3 {
	return gene.SurfaceNormal(p, t.SDF, 0.1)
}

func (t *TerrainGene) Animate(_ float64) {} // terrain không animate

func (t *TerrainGene) DNA() string {
	return fmt.Sprintf("∫(fbm,oct:%d,seed:%.0f)", t.Octaves, t.Seed)
}

func (t *TerrainGene) SimpleSDF(p gene.Vec3) float64 {
	// LOD: phẳng hơn khi xa
	h := gene.FBM((p.X-t.OriginX)/t.Scale, (p.Z-t.OriginZ)/t.Scale, 2, t.Seed)
	return p.Y - h*t.HeightScale
}
