// AppKit owns the walker so it keeps moving even when the handheld webview is hidden.
#import <AppKit/AppKit.h>
#import <math.h>

static const CGFloat walkingSize = 154;
static const NSInteger strideFrames = 8;
static const NSTimeInterval launchDuration = 0.72;

static CGFloat clamp(CGFloat value, CGFloat low, CGFloat high) {
    return MAX(low, MIN(high, value));
}

static NSPoint fitCenter(NSPoint p, NSRect area, CGFloat size) {
    CGFloat pad = size / 2;
    return NSMakePoint(clamp(p.x, NSMinX(area) + pad, NSMaxX(area) - pad),
                       clamp(p.y, NSMinY(area) + pad, NSMaxY(area) - pad));
}

@interface TamaWalkerView : NSView
@property(nonatomic, strong) NSImage *atlas;
@property(nonatomic) NSInteger pose;
@property(nonatomic) BOOL mirrored;
@end

@implementation TamaWalkerView
- (void)drawRect:(NSRect)dirty {
    NSGraphicsContext *graphics = NSGraphicsContext.currentContext;
    [graphics saveGraphicsState];
    NSRectFillUsingOperation(dirty, NSCompositingOperationClear);
    if (self.mirrored) {
        NSAffineTransform *flip = [NSAffineTransform transform];
        [flip translateXBy:self.bounds.size.width yBy:0];
        [flip scaleXBy:-1 yBy:1];
        [flip concat];
    }
    CGFloat cell = self.atlas.size.height;
    NSRect source = NSMakeRect(self.pose * cell, 0, cell, cell);
    [self.atlas drawInRect:self.bounds fromRect:source operation:NSCompositingOperationSourceOver fraction:1];
    [graphics restoreGraphicsState];
}
@end

@interface TamaWalk : NSObject
@property(nonatomic, strong) NSPanel *panel;
@property(nonatomic, strong) TamaWalkerView *sprite;
@property(nonatomic, strong) NSTimer *timer;
@property(nonatomic, strong) NSScreen *screen;
@property(nonatomic) NSPoint start, center, target;
@property(nonatomic) CGFloat initialSize, traveled;
@property(nonatomic) NSTimeInterval launch, last, restUntil;
@property(nonatomic) BOOL reduced, leftFacingArt;
@end

@implementation TamaWalk
- (void)chooseTarget:(NSRect)area {
    CGFloat roomLeft = self.center.x - NSMinX(area) - walkingSize / 2;
    CGFloat roomRight = NSMaxX(area) - self.center.x - walkingSize / 2;
    BOOL right = roomLeft < 110 || (roomRight >= 110 && arc4random_uniform(2));
    CGFloat dx = MIN(right ? roomRight : roomLeft, 170 + arc4random_uniform(201));
    CGFloat dy = (int)arc4random_uniform(181) - 90;
    self.target = fitCenter(NSMakePoint(self.center.x + (right ? dx : -dx), self.center.y + dy), area, walkingSize);
}

- (void)step {
    // Recover when the original display is disconnected or its usable area changes.
    if (![NSScreen.screens containsObject:self.screen]) self.screen = self.panel.screen ?: NSScreen.mainScreen;
    if (!self.screen) return;
    NSRect area = self.screen.visibleFrame;
    NSTimeInterval now = NSProcessInfo.processInfo.systemUptime;
    NSTimeInterval dt = MIN(0.08, now - self.last);
    self.last = now;
    CGFloat t = MIN(1, (now - self.launch) / launchDuration);
    CGFloat size = walkingSize;
    NSInteger pose = 0;
    NSPoint p = self.center;
    self.target = fitCenter(self.target, area, walkingSize);
    if (t < 1) {
        CGFloat ease = t * t * (3 - 2 * t);
        size = self.initialSize + (walkingSize - self.initialSize) * ease;
        p.x = self.start.x + (self.target.x - self.start.x) * ease;
        p.y = self.start.y + (self.target.y - self.start.y) * ease + (self.reduced ? 0 : sin(M_PI * t) * 68);
    } else {
        p = fitCenter(p, area, size);
        CGFloat dx = self.target.x - p.x, dy = self.target.y - p.y;
        CGFloat distance = hypot(dx, dy);
        if (distance < 1) {
            if (self.restUntil == 0) self.restUntil = now + 0.65 + arc4random_uniform(1200) / 1000.;
            if (now >= self.restUntil) {
                [self chooseTarget:area];
                self.restUntil = 0;
            }
        } else {
            CGFloat delta = MIN(distance, (self.reduced ? 50 : 84) * dt);
            p.x += delta * dx / distance;
            p.y += delta * dy / distance;
            self.traveled += delta;
            if (!self.reduced) pose = (NSInteger)(self.traveled / 5) % strideFrames;
        }
    }
    BOOL mirror = self.sprite.mirrored;
    CGFloat heading = self.target.x - p.x;
    if (fabs(heading) > 1) mirror = (heading > 0) == self.leftFacingArt;
    if (pose != self.sprite.pose || mirror != self.sprite.mirrored) {
        self.sprite.pose = pose;
        self.sprite.mirrored = mirror;
        self.sprite.needsDisplay = YES;
    }
    self.center = fitCenter(p, area, size);
    NSRect frame = NSMakeRect(round(self.center.x - size/2), round(self.center.y - size/2), size, size);
    if (!NSEqualRects(frame, self.panel.frame)) [self.panel setFrame:frame display:YES];
}
@end

static TamaWalk *walk;

// These functions are called exclusively from Tauri's main thread. The atlas is an
// eight-frame transparent strip rendered from the very same model shown on the LCD.
bool tama_walk_start(void *owner, const uint8_t *encoded, size_t count,
                     double x, double y, double size, bool leftFacingArt) {
    if (walk) return true;
    NSWindow *window = (__bridge NSWindow *)owner;
    NSScreen *screen = window.screen ?: NSScreen.mainScreen;
    if (!screen || !window.contentView || !isfinite(x) || !isfinite(y) || !isfinite(size)) return false;
    NSData *base64 = [NSData dataWithBytes:encoded length:count];
    NSData *data = [[NSData alloc] initWithBase64EncodedData:base64 options:0];
    NSBitmapImageRep *rep = data ? [NSBitmapImageRep imageRepWithData:data] : nil;
    if (!rep || rep.pixelsHigh != 256 || rep.pixelsWide != 256 * strideFrames) return false;
    NSImage *atlas = [[NSImage alloc] initWithSize:NSMakeSize(rep.pixelsWide, rep.pixelsHigh)];
    [atlas addRepresentation:rep];

    TamaWalk *next = [TamaWalk new];
    next.screen = screen;
    NSRect content = [window convertRectToScreen:window.contentLayoutRect];
    next.start = NSMakePoint(NSMinX(content) + x, NSMaxY(content) - y);
    next.initialSize = clamp(size, 80, 320);
    next.center = next.start;
    NSRect area = screen.visibleFrame;
    BOOL right = next.start.x < NSMidX(area);
    CGFloat exitX = right ? NSMaxX(window.frame) + walkingSize/2 : NSMinX(window.frame) - walkingSize/2;
    next.target = fitCenter(NSMakePoint(exitX, next.start.y - 62), area, walkingSize);
    next.reduced = NSWorkspace.sharedWorkspace.accessibilityDisplayShouldReduceMotion;
    next.leftFacingArt = leftFacingArt;
    next.launch = next.last = NSProcessInfo.processInfo.systemUptime;

    NSPanel *panel = [[NSPanel alloc] initWithContentRect:NSMakeRect(0, 0, walkingSize, walkingSize)
                                                styleMask:NSWindowStyleMaskBorderless | NSWindowStyleMaskNonactivatingPanel
                                                  backing:NSBackingStoreBuffered defer:NO];
    panel.title = @"tama96 desktop pet";
    panel.releasedWhenClosed = NO;
    panel.opaque = NO;
    panel.backgroundColor = NSColor.clearColor;
    panel.hasShadow = NO;
    panel.ignoresMouseEvents = YES;
    panel.hidesOnDeactivate = NO;
    panel.level = NSFloatingWindowLevel + 1;
    panel.collectionBehavior = NSWindowCollectionBehaviorIgnoresCycle | NSWindowCollectionBehaviorCanJoinAllSpaces;
    TamaWalkerView *sprite = [[TamaWalkerView alloc] initWithFrame:panel.contentView.bounds];
    sprite.atlas = atlas;
    sprite.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
    panel.contentView = sprite;
    next.sprite = sprite;
    next.panel = panel;
    walk = next;
    [next step];
    [panel orderFrontRegardless];
    NSTimer *timer = [NSTimer timerWithTimeInterval:1.0/30 repeats:YES block:^(NSTimer * _timer __attribute__((unused))) {
        [next step];
    }];
    timer.tolerance = 0.004;
    next.timer = timer;
    [NSRunLoop.mainRunLoop addTimer:timer forMode:NSRunLoopCommonModes];
    return true;
}

void tama_walk_stop(void) {
    [walk.timer invalidate];
    [walk.panel orderOut:nil];
    [walk.panel close];
    walk = nil;
}
