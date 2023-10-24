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