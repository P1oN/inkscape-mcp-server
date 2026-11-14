// Owned acceptance app. No client, engine, drawing or user preferences are opened.
#import <Cocoa/Cocoa.h>
int main(int argc,const char **argv) {@autoreleasepool {
    if(argc==3 && strcmp(argv[1],"--hold")==0){NSString *marker=[NSString stringWithUTF8String:argv[2]];for(int i=0;i<600 && ![[NSFileManager defaultManager] fileExistsAtPath:marker];i++)usleep(100000);return 0;}
    NSApplication *app=[NSApplication sharedApplication];[app setActivationPolicy:NSApplicationActivationPolicyProhibited];
    NSString *root=[[NSUserDefaults standardUserDefaults] stringForKey:@"InstallationDirectory"];
    if(!root)return 3;
    if([[NSFileManager defaultManager] fileExistsAtPath:[root stringByAppendingPathComponent:@"native-no-ack"]])return 5;
    NSTask *helper=[NSTask new];helper.executableURL=[NSURL fileURLWithPath:[[NSBundle mainBundle].bundlePath stringByAppendingPathComponent:@"Contents/Helpers/inkscape-mcp-launcher"]];helper.arguments=@[@"manager-prepare",@"--install-dir",root,@"--json"];
    NSPipe *out=[NSPipe pipe];helper.standardOutput=out;helper.standardError=[NSFileHandle fileHandleWithNullDevice];helper.standardInput=[NSFileHandle fileHandleWithNullDevice];
    NSError *error=nil;if(![helper launchAndReturnError:&error])return 4;
    NSData *response=[out.fileHandleForReading readDataToEndOfFile];[helper waitUntilExit];
    NSMutableDictionary *record=[NSMutableDictionary dictionaryWithDictionary:@{@"status":@(helper.terminationStatus),@"response":[[NSString alloc] initWithData:response encoding:NSUTF8StringEncoding] ?: @"",@"installation":root,@"channel":[[NSUserDefaults standardUserDefaults] stringForKey:@"UpdateChannel"] ?: @"missing"}];
    for(NSString *key in @[@"HOME",@"CODEX_HOME",@"CFFIXED_USER_HOME"])record[key]=NSProcessInfo.processInfo.environment[key] ?: @"missing";
    [[NSJSONSerialization dataWithJSONObject:record options:0 error:nil] writeToFile:[root stringByAppendingPathComponent:@"native-startup.json"] atomically:YES];
    return helper.terminationStatus;
}}
