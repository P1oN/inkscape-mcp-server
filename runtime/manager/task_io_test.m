#import "task_io.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, const char **argv) {
    @autoreleasepool {
        if(argc!=3)return 2;
        NSString *mode=@(argv[2]);
        if(strcmp(argv[1],"child")==0){
            if([@[@"ignore-term",@"blocked-input"] containsObject:mode]){signal(SIGTERM,SIG_IGN);puts("ready");fflush(stdout);for(;;)pause();}
            if([mode isEqualToString:@"held-pipe"]){pid_t child=fork();if(child==0){sleep(10);_exit(0);}printf("%d\n",child);return 0;}
            if([mode isEqualToString:@"oversize"]){char bytes[4096];memset(bytes,'x',sizeof(bytes));for(;;)if(write(STDOUT_FILENO,bytes,sizeof(bytes))<0)return 0;}
            puts("{\"ok\":true}");return 0;
        }
        NSTask *task=[NSTask new];task.executableURL=[NSURL fileURLWithPath:@(argv[0])];task.arguments=@[@"child",mode];
        NSPipe *pipe=[NSPipe pipe];task.standardOutput=pipe;NSPipe *input=[NSPipe pipe];task.standardInput=input;task.standardError=[NSFileHandle fileHandleWithNullDevice];
        if(![task launchAndReturnError:nil])return 3;
        // Child readiness avoids testing loader scheduling against the short budget.
        NSData *prefix=[pipe.fileHandleForReading readDataOfLength:1];
        __block BOOL current=YES;__block BOOL expired=NO;__block NSData *output=nil;
        NSTimeInterval start=NSProcessInfo.processInfo.systemUptime;
        managerTaskDeadline(task,0.3,0.2,^BOOL{return current;},^{expired=YES;});
        dispatch_group_t group=dispatch_group_create();
        NSData *request=[NSMutableData dataWithLength:[mode isEqualToString:@"blocked-input"] ? 1024*1024 : 1];
        dispatch_group_async(group,dispatch_get_global_queue(0,0),^{managerWriteInput(input.fileHandleForWriting,request,start+0.5);});
        dispatch_group_async(group,dispatch_get_global_queue(0,0),^{NSMutableData *captured=[prefix mutableCopy];[captured appendData:managerReadOutput(pipe.fileHandleForReading,task,start+0.5,nil)];output=captured;});
        dispatch_group_async(group,dispatch_get_global_queue(0,0),^{[task waitUntilExit];});
        dispatch_group_notify(group,dispatch_get_main_queue(),^{
            current=NO;
            BOOL hangs=[@[@"ignore-term",@"held-pipe",@"blocked-input"] containsObject:mode];
            if([mode isEqualToString:@"held-pipe"]){int child=[[[NSString alloc] initWithData:output encoding:NSUTF8StringEncoding] intValue];if(child<=0)exit(4);kill(child,SIGKILL);}
            if(expired!=hangs || NSProcessInfo.processInfo.systemUptime-start>2 || output.length>1024*1024+1)exit(5);
            if([mode isEqualToString:@"ignore-term"] && task.terminationStatus!=SIGKILL)exit(6);
            if([mode isEqualToString:@"normal"] && ![[NSJSONSerialization JSONObjectWithData:output options:0 error:nil][@"ok"] boolValue])exit(7);
            exit(0);
        });
        dispatch_main();
    }
}
