// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Device
import protocol Gemstone.GemDeviceStore
import GemstonePrimitives
import Preferences
import Primitives

public final class GemstoneDeviceStore: GemDeviceStore, @unchecked Sendable {
    private let preferences: Preferences

    public init(preferences: Preferences = .standard) {
        self.preferences = preferences
    }

    public func isRegistered() async throws -> Bool {
        preferences.isDeviceRegistered
    }

    public func setRegistered(registered: Bool) async throws {
        preferences.isDeviceRegistered = registered
    }

    public func getSubscriptionsVersion() async throws -> Int32 {
        preferences.subscriptionsVersion.asInt32
    }

    public func setSubscriptionsVersion(version: Int32) async throws {
        preferences.subscriptionsVersion = Int(version)
    }

    public func getPushedDevice() async throws -> Gemstone.Device? {
        preferences.pushedDevice
    }

    public func setPushedDevice(device: Gemstone.Device) async throws {
        preferences.pushedDevice = device
    }

    public func getPushedSubscriptions() async throws -> String? {
        preferences.pushedSubscriptions
    }

    public func setPushedSubscriptions(signature: String) async throws {
        preferences.pushedSubscriptions = signature
    }
}
