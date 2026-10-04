export function syncTextPreviews(model, container) {
  for (const pre of container.querySelectorAll('[data-preview-path]')) {
    const text = model.text(pre.dataset.previewPath);
    if (pre.textContent !== text) pre.textContent = text;
  }
}
const flowAdvice = {
  '1': '检查入口是否指向现存节点，以及节点 ID 是否合规、唯一。',
  '2': '检查显式边的两端节点，移除自环或重复边。',
  '3': '检查入口和显式边，使所有节点都能从入口到达；连线不会自动增加输入绑定。',
  '4': '至少保留一个没有出边的终点节点。',
  '5': '核对输入来源、引用输出与显式边的可达关系，并核对 required 声明。',
  '6': '为门槛节点填写非空说明。',
  '7': '核对引用文件是否存在，以及文件路径、大小和说明的 UTF-8 编码。',
  '8': '核对节点的 requires 与方法的宿主资源声明；工具不会安装资源。',
  '9': '人工执行者不得声明 tier；将档位设为「未声明」。',
  '10': '最终成果只能选择终点的必需项，输入与输出的结果名称不能重复。',
  'parse': '按引擎给出的字段或 TOML 行列核对声明类型与未知字段，修正后重新检查。',
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
    message: typeof error?.message === 'string' ? error.message : result.error ?? '检查失败。',
    code, rule, path, reason,
    next: code === 'FLOW_INVALID' ? flowAdvice[rule] ?? '按上述字段和原因核对当前 Flow，修改后重新检查。' : '按上述字段和原因核对当前文件，修改后重新检查。',
    fileNote: explicitFile ? `引擎给出的文件：${explicitFile}` : '引擎未提供准确文件。下面列出当前定义文件候选，请打开后核对上述字段。',
    candidates: explicitFile ? [explicitFile] : ['workbook.toml', ...flowPaths],
  };
}
