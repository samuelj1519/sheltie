# 跨节点同号Occurrence批准独立复核

Reviewer：/root/spec_review，未参与实施、未改文件。新CLI oracle合格：独立first→second→done，前二节点gate；真实submit/approve后合法status为正例。反例只删除second#1批准，保留first#1，其他事实不变且累计受阻必要界仍合法；期望来自GF-12的node+occurrence身份，不用生产校验器生成。status/begin应STORE_CORRUPT、坏state原样、目标request无登记。精确AND→OR反实现被捕获exit100，恢复原source SHA后绿，证明检出真实差异。

旧3a9f输入的完整core及中断runtime原文归档不复用；新c31dcb完整mutation与最终disposition仍待实际运行，isolated捕获不能代替新完整候选通过。
