import Foundation
import ObjcFunc

/// Swift wrapper for the ObjcFunc Objective-C class
@objc public class ObjcFuncSwift: NSObject {
    
    // MARK: - Properties
    private let objcFunc: ObjcFunc
    
    /// The name property from the underlying ObjcFunc
    public var name: String {
        get { return objcFunc.name }
        set { objcFunc.name = newValue }
    }
    
    /// The count property from the underlying ObjcFunc
    public var count: Int {
        get { return objcFunc.count }
        set { objcFunc.count = newValue }
    }
    
    /// The items array from the underlying ObjcFunc
    public var items: [String] {
        return objcFunc.items as? [String] ?? []
    }
    
    /// The enabled property from the underlying ObjcFunc
    public var isEnabled: Bool {
        get { return objcFunc.isEnabled }
        set { objcFunc.isEnabled = newValue }
    }
    
    // MARK: - Initializers
    
    /// Default initializer
    public override init() {
        self.objcFunc = ObjcFunc()
        super.init()
    }
    
    /// Initializer with name and count
    /// - Parameters:
    ///   - name: The name for the instance
    ///   - count: The initial count value
    public init(name: String, count: Int) {
        self.objcFunc = ObjcFunc(name: name, count: count)
        super.init()
    }
    
    /// Internal initializer with existing ObjcFunc instance
    /// - Parameter objcFunc: An existing ObjcFunc instance
    internal init(objcFunc: ObjcFunc) {
        self.objcFunc = objcFunc
        super.init()
    }
    
    // MARK: - Instance Methods
    
    /// Performs an action using the underlying ObjcFunc
    public func performAction() {
        objcFunc.performAction()
    }
    
    /// Gets a formatted description from the underlying ObjcFunc
    /// - Returns: A formatted description string
    public func getFormattedDescription() -> String {
        return objcFunc.getFormattedDescription()
    }
    
    /// Adds an item to the items array
    /// - Parameter item: The item to add
    public func addItem(_ item: String) {
        objcFunc.addItem(item)
    }
    
    /// Removes an item from the items array
    /// - Parameter item: The item to remove
    public func removeItem(_ item: String) {
        objcFunc.removeItem(item)
    }
    
    /// Gets the count of items in the array
    /// - Returns: The number of items
    public func getItemCount() -> Int {
        return objcFunc.getItemCount()
    }
    
    /// Increments the count value
    public func incrementCount() {
        objcFunc.incrementCount()
    }
    
    /// Decrements the count value
    public func decrementCount() {
        objcFunc.decrementCount()
    }
    
    /// Resets all properties to their default values
    public func reset() {
        objcFunc.reset()
    }
    
    // MARK: - Class Methods
    
    /// Gets the default shared instance wrapped in Swift
    /// - Returns: A Swift wrapper around the default ObjcFunc instance
    public static func defaultInstance() -> ObjcFuncSwift {
        let objcInstance = ObjcFunc.defaultInstance()
        return ObjcFuncSwift(objcFunc: objcInstance)
    }
    
    /// Gets the class description
    /// - Returns: A description of the class
    public static func classDescription() -> String {
        return ObjcFunc.classDescription()
    }
    
    /// Validates a name string
    /// - Parameter name: The name to validate
    /// - Returns: True if the name is valid, false otherwise
    public static func validateName(_ name: String) -> Bool {
        return ObjcFunc.validateName(name)
    }
}

// MARK: - Swift-specific Extensions

extension ObjcFuncSwift {
    
    /// Swift-style computed property for description
    public override var description: String {
        return getFormattedDescription()
    }
    
    /// Subscript access to items by index
    /// - Parameter index: The index of the item
    /// - Returns: The item at the specified index, or nil if out of bounds
    public subscript(index: Int) -> String? {
        let itemsArray = items
        guard index >= 0 && index < itemsArray.count else { return nil }
        return itemsArray[index]
    }
    
    /// Adds multiple items at once
    /// - Parameter items: An array of items to add
    public func addItems(_ items: [String]) {
        items.forEach { addItem($0) }
    }
    
    /// Removes all items that match the predicate
    /// - Parameter predicate: A closure that returns true for items to remove
    public func removeItems(where predicate: (String) -> Bool) {
        let itemsToRemove = items.filter(predicate)
        itemsToRemove.forEach { removeItem($0) }
    }
}
