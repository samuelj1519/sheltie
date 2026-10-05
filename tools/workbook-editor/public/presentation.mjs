export function syncTextPreviews(model, container) {
  for (const pre of container.querySelectorAll('[data-preview-path]')) {
    const text = model.text(pre.dataset.previewPath);
    if (pre.textContent !== text) pre.textContent = text;
  }
}
const flowAdvice = {
  '1': 'Check that the entry names an existing node and all node IDs are valid and unique.',
  '2': 'Check edge endpoints and remove self-loops or duplicate edges.',
  '3': 'Check the entry and explicit edges so every node is reachable; connecting nodes does not add input bindings.',
  '4': 'Keep at least one terminal node with no outgoing edges.',
  '5': 'Check input sources, referenced outputs, explicit-edge reachability, and required declarations.',
  '6': 'Provide nonempty instructions for gate nodes.',
  '7': 'Check referenced files, paths, sizes, and UTF-8 instruction encoding.',
  '8': 'Check node requires against Workbook host-resource declarations; the editor does not install resources.',
  '9': 'Human executors must not declare tier; select Not declared.',
  '10': 'Final results must select required terminal items, with unique names across inputs and outputs.',
  'parse': 'Check declaration types and unknown fields using the engine field or TOML line/column, then retry.',
};
export function describeFailure(result, flowPaths) {
  const error = result.engineError;
  const detail = error?.detail;
  const code = typeof error?.code === 'string' ? error.code : '';
  const rule = typeof detail?.rule === 'string' ? detail.rule : '';
  const path = typeof detail?.path === 'string' ? detail.path : '';
  const reason = typeof detail?.reason === 'string' ? detail.reason : '';
  const explicitFile = typeof detail?.file === 'string' ? detail.file : '';
  return {
    message: typeof error?.message === 'string' ? error.message : result.error ?? 'Check failed.',
    code, rule, path, reason,
    next: code === 'FLOW_INVALID' ? flowAdvice[rule] ?? 'Check the current Flow against the fields and reason above, then run the check again.' : 'Check the current file against the fields and reason above, then run the check again.',
    fileNote: explicitFile ? `File identified by the engine: ${explicitFile}` : 'The engine did not identify an exact file. Open a candidate definition below and check the fields above.',
    candidates: explicitFile ? [explicitFile] : ['workbook.toml', ...flowPaths],
  };
}
