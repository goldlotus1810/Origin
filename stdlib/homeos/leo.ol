// homeos/leo.ol — LeoAI: bộ não (học, hiểu, sắp xếp, nhớ, LẬP TRÌNH)

pub fn leo_process(text, emotion, context) {
  // 1. Classify intent
  let intent = estimate_intent(text, emotion);

  // 2. Crisis → gate handles
  if intent == "crisis" {
    let gate = gate_check(text);
    return gate;
  }

  // 3. Command → pass through
  if intent == "command" {
    return { action: "command", text: text };
  }

  // 4. Learn/Chat → full pipeline
  let result = process_one(text, emotion, context);
  result.intent = intent;
  return result;
}

pub fn leo_dream(context) {
  return context_dream(context);
}

pub fn leo_express(hash, stm) {
  // Express observation as Olang molecular literal
  let i = 0;
  while i < len(stm.entries) {
    if stm.entries[i].hash == hash {
      let mol = stm.entries[i].mol;
      return "{ S=" + to_string(mol.s) + " R=" + to_string(mol.r) +
             " V=" + to_string(mol.v) + " A=" + to_string(mol.a) +
             " T=" + to_string(mol.t) + " }";
    }
    i = i + 1;
  }
  return "{ }";
}
