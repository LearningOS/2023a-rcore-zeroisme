### 编程作业
1. 在`TaskControlBlock`中添加`started_time`和`syscall_times`数组
2. 在task创建的时候，将`started_time`设置为`get_time()`
3. 在发生syscall时，将`syscall_times[SYSCALL_ID]+=1`
4. 在TaskManager添加一个函数，获取当前任务
5. `sys_task_info`实现分以下几步
    1. 将task_info任务状态设置为`Running`
    1. 先`get_time()`获取当前时间
    2. 使用获取的当前时间 - started_time得到elapsed_time
    3. 转换为ms赋给参数task_info
    4. copy syscall_times到task_info中的syscall_times
    5. 返回0

### 简答作业
1. 
    1. ch2b_bad_address.rs: 访问非法内存地址，PageFault
    2. ch2b_bad_instructions.rs: U模式下执行S模式下指令：IllegalInstruction
    3. ch2b_bad_register.rs: U模式下访问S模式下的寄存器：IllegalInstruction

2. 
	1. a0的值是kernel stack sp, __restore的两种使用场景： 1）任务切换后返回，恢复上下文 2）创建新任务，指定返回地址
	2. 读取sstatus, sepc, sscratch寄存器放入临时寄存器t0,t1,t2， sstatus跟踪处理器运行状态, sepc是sret的返回地址，sscratch是用来恢复用户栈
	3. x2是sp寄存器，需要将其它寄存器恢复后，再恢复sp寄存器。x4未使用，不需要恢复
	4. 切换sp为用户栈，sscratch为kernel stack
	5. 发生状态切换是sret, 指令会进入用户态是因为sstatus寄存器中spp设置为了SPP::User用户模式
    6. sp 保存 kernel stack, sscratch 保存 user stack
	7. 通过ecall指令从 U 态进入 S 态

### 荣誉准则
在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
参考资料：
https://learningos.cn/rCore-Tutorial-Guide-2023A/chapter3/index.html
https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/index.html

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。