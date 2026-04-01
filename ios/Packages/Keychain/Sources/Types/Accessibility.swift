// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum Accessibility: Sendable {
    /**
     Item data can only be accessed
     while the device is unlocked. This is recommended for items that only
     need be accesible while the application is in the foreground. Items
     with this attribute will migrate to a new device when using encrypted
     backups.
     */
    case whenUnlocked

    /**
     Item data can only be
     accessed once the device has been unlocked after a restart. This is
     recommended for items that need to be accesible by background
     applications. Items with this attribute will migrate to a new device
     when using encrypted backups.
     */
    case afterFirstUnlock

    /**
     Item data can
     only be accessed while the device is unlocked. This class is only
     available if a passcode is set on the device. This is recommended for
     items that only need to be accessible while the application is in the
     foreground. Items with this attribute will never migrate to a new
     device, so after a backup is restored to a new device, these items
     will be missing. No items can be stored in this class on devices
     without a passcode. Disabling the device passcode will cause all
     items in this class to be deleted.
     */
    @available(iOS 8.0, OSX 10.10, *)
    case whenPasscodeSetThisDeviceOnly

    /**
     Item data can only
     be accessed while the device is unlocked. This is recommended for items
     that only need be accesible while the application is in the foreground.
     Items with this attribute will never migrate to a new device, so after
     a backup is restored to a new device, these items will be missing.
     */
    case whenUnlockedThisDeviceOnly //

    /**
     Item data can
     only be accessed once the device has been unlocked after a restart.
     This is recommended for items that need to be accessible by background
     applications. Items with this attribute will never migrate to a new
     device, so after a backup is restored to a new device these items will
     be missing.
     */
    case afterFirstUnlockThisDeviceOnly
}

extension Accessibility: RawRepresentable, CustomStringConvertible {
    public init?(rawValue: String) {
        if #available(OSX 10.10, *) {
            switch rawValue {
            case String(kSecAttrAccessibleWhenUnlocked):
                self = .whenUnlocked
            case String(kSecAttrAccessibleAfterFirstUnlock):
                self = .afterFirstUnlock
            case String(kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly):
                self = .whenPasscodeSetThisDeviceOnly
            case String(kSecAttrAccessibleWhenUnlockedThisDeviceOnly):
                self = .whenUnlockedThisDeviceOnly
            case String(kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly):
                self = .afterFirstUnlockThisDeviceOnly
            default:
                return nil
            }
        } else {
            switch rawValue {
            case String(kSecAttrAccessibleWhenUnlocked):
                self = .whenUnlocked
            case String(kSecAttrAccessibleAfterFirstUnlock):
                self = .afterFirstUnlock
            case String(kSecAttrAccessibleWhenUnlockedThisDeviceOnly):
                self = .whenUnlockedThisDeviceOnly
            case String(kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly):
                self = .afterFirstUnlockThisDeviceOnly
            default:
                return nil
            }
        }
    }

    public var rawValue: String {
        switch self {
        case .whenUnlocked:
            String(kSecAttrAccessibleWhenUnlocked)
        case .afterFirstUnlock:
            String(kSecAttrAccessibleAfterFirstUnlock)
        case .whenPasscodeSetThisDeviceOnly:
            if #available(OSX 10.10, *) {
                String(kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly)
            } else {
                fatalError("'Accessibility.WhenPasscodeSetThisDeviceOnly' is not available on this version of OS.")
            }
        case .whenUnlockedThisDeviceOnly:
            String(kSecAttrAccessibleWhenUnlockedThisDeviceOnly)
        case .afterFirstUnlockThisDeviceOnly:
            String(kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly)
        }
    }

    public var description: String {
        switch self {
        case .whenUnlocked:
            "WhenUnlocked"
        case .afterFirstUnlock:
            "AfterFirstUnlock"
        case .whenPasscodeSetThisDeviceOnly:
            "WhenPasscodeSetThisDeviceOnly"
        case .whenUnlockedThisDeviceOnly:
            "WhenUnlockedThisDeviceOnly"
        case .afterFirstUnlockThisDeviceOnly:
            "AfterFirstUnlockThisDeviceOnly"
        }
    }
}
