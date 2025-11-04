@plugin.rs (1-662)

- this_compilation 会在编译模块之前开始一次， 所以我在这里直接 glob 的所有的文件进行 lint。如果是处于开发模式下的话，每一次的代码变更也会触发一次 this_compilation。 需要注意的是通常 this_compilation 的执行是按照整个编译器的执行来决定的， 而不是说 一个 module 执行一次
- succeed_module 的话是每个 module 在编译成功之后会执行， 也就是说如果我有 100 个 module， 那么就是会执行 100 次。

我其实想实现的需求是

- 在用户启动开发模式下的时候，只有第一次启动的时候 执行 this_compilation 里面的全量 global， 这次启动的时候因为也走了编译流程嘛，所以理论上每一个 module 也都会走一次succeed_module ， 这样的话其实就造成了重复 lint ， 我不想这样。
- 而在后续的用户代码变更的热更新中，其实我是不需要全量的 glob 的， 理论上就是 按照 succeed_module 变更了什么文件就执行哪个文件的 lint 就行了。
- 我也是希望在全局有 cache 的 hashmap 去存储有问题的 lint 文件的相关信息的
