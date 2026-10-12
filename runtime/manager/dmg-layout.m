// Development-only fixed Finder layout writer. Never included in the app or MCP surface.
// A bounded, single-leaf DS_Store (Bud1 allocator + DSDB B-tree), not a general editor.
#import <Foundation/Foundation.h>
#import <CoreServices/CoreServices.h>

static void require(BOOL ok, NSString *message) {
    if (!ok) { fprintf(stderr, "%s\n", message.UTF8String); exit(1); }
}
static void word(NSMutableData *data, uint32_t value) {
    uint32_t big = CFSwapInt32HostToBig(value); [data appendBytes:&big length:4];
}
static void at(NSMutableData *data, NSUInteger offset, uint32_t value) {
    uint32_t big = CFSwapInt32HostToBig(value); [data replaceBytesInRange:NSMakeRange(offset,4) withBytes:&big];
}
static NSData *plist(NSDictionary *value) {
    NSError *error=nil;
    NSData *data=[NSPropertyListSerialization dataWithPropertyList:value format:NSPropertyListBinaryFormat_v1_0 options:0 error:&error];
    require(data!=nil, error.localizedDescription); return data;
}
static void record(NSMutableData *data, NSString *name, const char *code, const char *type, NSData *value) {
    word(data, (uint32_t)name.length);
    [data appendData:[name dataUsingEncoding:NSUTF16BigEndianStringEncoding]];
    [data appendBytes:code length:4]; [data appendBytes:type length:4];
    if (strcmp(type,"blob")==0) word(data,(uint32_t)value.length);
    [data appendData:value];
}
static NSData *location(uint32_t x, uint32_t y) {
    NSMutableData *data=[NSMutableData data]; word(data,x); word(data,y); word(data,0xffffff00); word(data,0); return data;
}
static FSRef ref(NSString *path) {
    FSRef result; require(FSPathMakeRef((const UInt8 *)path.fileSystemRepresentation,&result,NULL)==noErr,@"Cannot reference DMG resource"); return result;
}
static NSData *backgroundAlias(NSString *mount) {
    FSRef base=ref(mount), target=ref([mount stringByAppendingPathComponent:@"Inkscape MCP Manager.app/Contents/Resources/installer-background.tiff"]);
    AliasHandle alias=NULL;
    require(FSNewAlias(&base,&target,&alias)==noErr,@"Cannot create relative background alias");
    NSData *data=[NSData dataWithBytes:*alias length:(NSUInteger)GetHandleSize((Handle)alias)];
    DisposeHandle((Handle)alias); return data;
}
static void writeLayout(NSString *mount) {
    NSDictionary *window=@{@"ShowToolbar":@NO,@"ShowSidebar":@NO,@"ContainerShowSidebar":@NO,@"ShowStatusBar":@NO,@"ShowPathbar":@NO,@"ShowTabView":@NO,@"SidebarWidth":@0,@"WindowBounds":@"{{160, 140}, {720, 512}}"};
    NSDictionary *icons=@{@"viewOptionsVersion":@1,@"backgroundType":@2,@"backgroundImageAlias":backgroundAlias(mount),@"backgroundColorRed":@1.0,@"backgroundColorGreen":@1.0,@"backgroundColorBlue":@1.0,@"iconSize":@160.0,@"textSize":@13.0,@"gridSpacing":@100.0,@"gridOffsetX":@0.0,@"gridOffsetY":@0.0,@"labelOnBottom":@YES,@"showItemInfo":@NO,@"showIconPreview":@YES,@"arrangeBy":@"none"};
    // Records sorted by case-insensitive filename, then the four-byte property code.
    NSMutableData *leaf=[NSMutableData data]; word(leaf,0); word(leaf,6);
    record(leaf,@".","bwsp","blob",plist(window));
    record(leaf,@".","icvp","blob",plist(icons));
    record(leaf,@".","vSrn","long",[NSData dataWithBytes:"\0\0\0\1" length:4]);
    record(leaf,@".","vstl","type",[NSData dataWithBytes:"icnv" length:4]);
    record(leaf,@"Applications","Iloc","blob",location(515,247));
    record(leaf,@"Inkscape MCP Manager.app","Iloc","blob",location(205,247));
    require(leaf.length<=8192,@"Finder layout exceeds its single-page bound");
    NSMutableData *store=[NSMutableData dataWithLength:16388];
    at(store,0,1); [store replaceBytesInRange:NSMakeRange(4,4) withBytes:"Bud1"];
    at(store,8,2048); at(store,12,2048); at(store,16,2048); at(store,20,0x100c);
    // Block 0: allocator at 2048, block 1: master at 64, block 2: leaf at 8192.
    at(store,68,2); at(store,72,0); at(store,76,6); at(store,80,1); at(store,84,8192);
    NSMutableData *allocator=[NSMutableData data]; word(allocator,3); word(allocator,0);
    word(allocator,2048|11); word(allocator,64|5); word(allocator,8192|13);
    [allocator increaseLengthBy:253*4];
    word(allocator,1); unsigned char length=4; [allocator appendBytes:&length length:1];
    [allocator appendBytes:"DSDB" length:4]; word(allocator,1);
    for (unsigned width=0;width<32;width++) {
        if (width==5) { word(allocator,2); word(allocator,32); word(allocator,96); }
        else if ((width>=7 && width<=10) || width==12 || (width>=14 && width<=30)) { word(allocator,1); word(allocator,1U<<width); }
        else word(allocator,0);
    }
    require(allocator.length<=2048,@"Allocator overflow");
    [store replaceBytesInRange:NSMakeRange(2052,allocator.length) withBytes:allocator.bytes];
    [store replaceBytesInRange:NSMakeRange(8196,leaf.length) withBytes:leaf.bytes];
    NSError *error=nil;
    require([store writeToFile:[mount stringByAppendingPathComponent:@".DS_Store"] options:NSDataWritingWithoutOverwriting error:&error],error.localizedDescription);
}
static uint32_t takeWord(NSData *data, NSUInteger *cursor) {
    require(*cursor+4<=data.length,@"Truncated Finder metadata");
    uint32_t big; [data getBytes:&big range:NSMakeRange(*cursor,4)]; *cursor+=4; return CFSwapInt32BigToHost(big);
}
static NSData *take(NSData *data, NSUInteger *cursor, NSUInteger size) {
    require(size<=8192 && *cursor+size<=data.length,@"Oversized or truncated Finder record");
    NSData *result=[data subdataWithRange:NSMakeRange(*cursor,size)]; *cursor+=size; return result;
}
static void verifyLayout(NSString *mount) {
    NSData *data=[NSData dataWithContentsOfFile:[mount stringByAppendingPathComponent:@".DS_Store"]];
    require(data.length==16388,@"Unexpected Finder metadata size");
    NSUInteger cursor=0; require(takeWord(data,&cursor)==1,@"Invalid DS_Store version");
    require(memcmp(take(data,&cursor,4).bytes,"Bud1",4)==0,@"Invalid allocator header");
    require(takeWord(data,&cursor)==2048 && takeWord(data,&cursor)==2048 && takeWord(data,&cursor)==2048,@"Invalid allocator bounds");
    cursor=8196; require(takeWord(data,&cursor)==0 && takeWord(data,&cursor)==6,@"Invalid B-tree leaf");
    NSMutableDictionary *records=[NSMutableDictionary new];
    for (unsigned i=0;i<6;i++) {
        uint32_t units=takeWord(data,&cursor);
        NSString *name=[[NSString alloc] initWithData:take(data,&cursor,(NSUInteger)units*2) encoding:NSUTF16BigEndianStringEncoding];
        NSString *code=[[NSString alloc] initWithData:take(data,&cursor,4) encoding:NSASCIIStringEncoding];
        NSString *type=[[NSString alloc] initWithData:take(data,&cursor,4) encoding:NSASCIIStringEncoding];
        require(name!=nil && code!=nil && type!=nil,@"Invalid record encoding");
        NSUInteger size=[type isEqualToString:@"blob"] ? takeWord(data,&cursor) : 4;
        records[[NSString stringWithFormat:@"%@/%@",name,code]]=take(data,&cursor,size);
    }
    NSDictionary *icons=[NSPropertyListSerialization propertyListWithData:records[@"./icvp"] options:0 format:NULL error:nil];
    NSDictionary *window=[NSPropertyListSerialization propertyListWithData:records[@"./bwsp"] options:0 format:NULL error:nil];
    require([icons[@"iconSize"] isEqual:@160] && [icons[@"arrangeBy"] isEqual:@"none"] && [icons[@"backgroundType"] isEqual:@2],@"Invalid icon view options");
    require([window[@"WindowBounds"] isEqual:@"{{160, 140}, {720, 512}}"] && [window[@"ShowToolbar"] isEqual:@NO],@"Invalid window options");
    require([records[@"Applications/Iloc"] isEqual:location(515,247)] && [records[@"Inkscape MCP Manager.app/Iloc"] isEqual:location(205,247)],@"Invalid icon positions");
    NSData *raw=icons[@"backgroundImageAlias"]; require([raw isKindOfClass:NSData.class] && raw.length<8192,@"Invalid background alias");
    AliasHandle alias=(AliasHandle)NewHandle((Size)raw.length); require(alias!=NULL,@"Alias allocation failed"); memcpy(*alias,raw.bytes,raw.length);
    FSRef base=ref(mount), resolved; Boolean changed=false;
    require(FSResolveAlias(&base,alias,&resolved,&changed)==noErr,@"Background alias does not resolve after remount"); DisposeHandle((Handle)alias);
    UInt8 path[4096]; require(FSRefMakePath(&resolved,path,sizeof(path))==noErr,@"Background path cannot be read");
    NSString *actual=[[NSFileManager defaultManager] stringWithFileSystemRepresentation:(const char *)path length:strlen((const char *)path)];
    NSString *expected=[mount stringByAppendingPathComponent:@"Inkscape MCP Manager.app/Contents/Resources/installer-background.tiff"];
    NSSet *entries=[NSSet setWithArray:[[NSFileManager defaultManager] contentsOfDirectoryAtPath:mount error:nil]];
    require([entries containsObject:@"Inkscape MCP Manager.app"] && [entries containsObject:@"Applications"] && ![entries containsObject:@".background"],@"Unexpected installer root layout");
    require([[actual stringByResolvingSymlinksInPath] isEqualToString:[expected stringByResolvingSymlinksInPath]],[NSString stringWithFormat:@"Background alias escaped mounted image: %@ != %@",actual,expected]);
    NSString *link=[[NSFileManager defaultManager] destinationOfSymbolicLinkAtPath:[mount stringByAppendingPathComponent:@"Applications"] error:nil];
    require([link isEqual:@"/Applications"],@"Invalid Applications shortcut");
    puts("Finder metadata, icon positions, Applications shortcut and remounted background alias verified");
}
int main(int argc, const char *argv[]) {
    @autoreleasepool {
        require(argc==3,@"Usage: dmg-layout write|verify PRIVATE_MOUNT");
        NSString *mount=[[NSString stringWithUTF8String:argv[2]] stringByResolvingSymlinksInPath];
        require([mount isAbsolutePath] && ![mount isEqual:@"/"],@"Expected private mounted image");
        if (strcmp(argv[1],"write")==0) writeLayout(mount);
        else if (strcmp(argv[1],"verify")==0) verifyLayout(mount);
        else require(NO,@"Unknown layout operation");
    }
}
