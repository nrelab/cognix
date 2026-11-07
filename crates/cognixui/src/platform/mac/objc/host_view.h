#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>

@interface NSPasteboard (Cognix)
- (NSArray *)getFilePaths;
@end

/// CognixHostView is the Content view of a Cognix window.
// It is backed by a Metal CALayer.
@interface CognixHostView : NSView <CALayerDelegate, NSTextInputClient>
- (CognixHostView *)initWithFrame:(NSRect)frame
                    metalDevice:(id)metalDevice
             enableTitlebarDrag:(BOOL)enableTitlebarDrag
                       testMode:(BOOL)testMode;
- (void)setAsyncCallback:(BOOL)shouldAsync;
- (BOOL)keyDownImpl:(NSEvent *)event;
@end
