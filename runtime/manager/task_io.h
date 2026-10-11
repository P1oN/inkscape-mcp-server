#import <Foundation/Foundation.h>
#import <poll.h>
#import <signal.h>
#import <errno.h>
#import <unistd.h>
#import <fcntl.h>

// The main queue owns the operation identity and timeout state. A completed task
// may still have a descendant holding its pipes; readers have their own deadline.
static void managerTaskDeadline(NSTask *task, NSTimeInterval seconds, NSTimeInterval grace,
                                BOOL (^current)(void), void (^expired)(void)) {
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW,(int64_t)(seconds*NSEC_PER_SEC)),dispatch_get_main_queue(),^{
        if(!current())return;
        expired();
        if(task.running)[task terminate];
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW,(int64_t)(grace*NSEC_PER_SEC)),dispatch_get_main_queue(),^{
            if(current() && task.running)kill(task.processIdentifier,SIGKILL);
        });
    });
}

static NSData *managerReadOutput(NSFileHandle *file, NSTask *task, NSTimeInterval deadline,
                                 void (^lineHandler)(NSData *)) {
    NSMutableData *data=[NSMutableData new];NSMutableData *pending=[NSMutableData new];
    while(NSProcessInfo.processInfo.systemUptime<deadline){
        struct pollfd descriptor={.fd=file.fileDescriptor,.events=POLLIN};
        int ready=poll(&descriptor,1,100);
        if(ready<0){if(errno==EINTR)continue;break;}
        if(!ready)continue;
        char bytes[4096];ssize_t count=read(descriptor.fd,bytes,sizeof(bytes));
        if(count<0){if(errno==EINTR)continue;break;}
        if(!count)break;
        NSData *part=[NSData dataWithBytes:bytes length:(NSUInteger)count];
        if(data.length+part.length>1024*1024){if(task.running)kill(task.processIdentifier,SIGKILL);break;}
        [data appendData:part];
        if(lineHandler){[pending appendData:part];while(YES){NSRange end=[pending rangeOfData:[NSData dataWithBytes:"\n" length:1] options:0 range:NSMakeRange(0,pending.length)];if(end.location==NSNotFound)break;if(end.location<=4096)lineHandler([pending subdataWithRange:NSMakeRange(0,end.location)]);[pending replaceBytesInRange:NSMakeRange(0,end.location+1) withBytes:NULL length:0];}if(pending.length>4096)[pending setLength:0];}
    }
    [file closeFile];
    return data;
}

// Input is written off the main queue so a helper that never reads its request
// cannot prevent the watchdog or recovery UI from running.
static void managerWriteInput(NSFileHandle *file, NSData *data, NSTimeInterval deadline) {
    int fd=file.fileDescriptor;
    fcntl(fd,F_SETNOSIGPIPE,1);
    int flags=fcntl(fd,F_GETFL);
    if(flags<0 || fcntl(fd,F_SETFL,flags|O_NONBLOCK)<0){[file closeFile];return;}
    NSUInteger offset=0;
    while(offset<data.length && NSProcessInfo.processInfo.systemUptime<deadline){
        struct pollfd descriptor={.fd=fd,.events=POLLOUT};
        int ready=poll(&descriptor,1,100);
        if(ready<0){if(errno==EINTR)continue;break;}
        if(!ready)continue;
        ssize_t count=write(fd,(const char *)data.bytes+offset,MIN((NSUInteger)4096,data.length-offset));
        if(count<0){if(errno==EINTR || errno==EAGAIN)continue;break;}
        if(!count)break;
        offset+=(NSUInteger)count;
    }
    [file closeFile];
}
