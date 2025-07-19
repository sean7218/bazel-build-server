#import "ObjcFunc.h"

@implementation ObjcFunc

#pragma mark - Initializers

- (instancetype)init {
    return [self initWithName:@"Default" count:0];
}

- (instancetype)initWithName:(NSString *)name count:(NSInteger)count {
    self = [super init];
    if (self) {
        _name = [name copy];
        _count = count;
        _value = 0.0;
        _items = [[NSMutableArray alloc] init];
        _enabled = YES;
    }
    return self;
}

#pragma mark - Instance Methods

- (void)performAction {
    if (self.enabled) {
        NSLog(@"Performing action for %@ with count: %ld", self.name, (long)self.count);
        [self incrementCount];
    } else {
        NSLog(@"Action disabled for %@", self.name);
    }
}

- (NSString *)getFormattedDescription {
    return [NSString stringWithFormat:@"ObjcFunc: %@ (count: %ld, value: %.2f, items: %ld, enabled: %@)",
            self.name,
            (long)self.count,
            self.value,
            (long)[self getItemCount],
            self.enabled ? @"YES" : @"NO"];
}

- (void)addItem:(NSString *)item {
    if (item && item.length > 0) {
        [self.items addObject:item];
        NSLog(@"Added item: %@", item);
    }
}

- (void)removeItem:(NSString *)item {
    if ([self.items containsObject:item]) {
        [self.items removeObject:item];
        NSLog(@"Removed item: %@", item);
    }
}

- (NSInteger)getItemCount {
    return self.items.count;
}

- (void)incrementCount {
    self.count++;
    self.value += 1.5;
}

- (void)decrementCount {
    if (self.count > 0) {
        self.count--;
        self.value -= 1.5;
    }
}

- (void)reset {
    self.count = 0;
    self.value = 0.0;
    [self.items removeAllObjects];
    self.enabled = YES;
    NSLog(@"Reset %@", self.name);
}

#pragma mark - Class Methods

+ (instancetype)defaultInstance {
    static ObjcFunc *sharedInstance = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        sharedInstance = [[ObjcFunc alloc] initWithName:@"SharedInstance" count:10];
    });
    return sharedInstance;
}

+ (NSString *)classDescription {
    return @"ObjcFunc is a utility class for demonstration purposes with various member variables and methods.";
}

+ (BOOL)validateName:(NSString *)name {
    return name != nil && name.length > 0 && name.length <= 50;
}

@end