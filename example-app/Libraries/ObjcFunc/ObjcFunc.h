#import <Foundation/Foundation.h>

NS_ASSUME_NONNULL_BEGIN

@interface ObjcFunc : NSObject

// Member variables (properties)
@property (nonatomic, strong) NSString *name;
@property (nonatomic, assign) NSInteger count;
@property (nonatomic, assign) CGFloat value;
@property (nonatomic, strong) NSMutableArray<NSString *> *items;
@property (nonatomic, assign, getter=isEnabled) BOOL enabled;

// Initializers
- (instancetype)init;
- (instancetype)initWithName:(NSString *)name count:(NSInteger)count;

// Instance methods
- (void)performAction;
- (NSString *)getFormattedDescription;
- (void)addItem:(NSString *)item;
- (void)removeItem:(NSString *)item;
- (NSInteger)getItemCount;
- (void)incrementCount;
- (void)decrementCount;
- (void)reset;

// Class methods
+ (instancetype)defaultInstance;
+ (NSString *)classDescription;
+ (BOOL)validateName:(NSString *)name;

@end

NS_ASSUME_NONNULL_END