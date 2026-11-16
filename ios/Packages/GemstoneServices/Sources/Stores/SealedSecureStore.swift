// Copyright (c). Gem Wallet. All rights reserved.

import CryptoKit
import Foundation
import Gemstone
import Keychain
import Primitives

public final class GemstoneSealedSecureStore: GemSecureStore, @unchecked Sendable {
    private static let sealingKey = "sealing_key"
    private static let lock = NSLock()

    private let keychain: Keychain
    private let namespace: String
    private let directory: URL

    public init(
        namespace: String,
        keychain: Keychain = KeychainDefault(),
        directory: URL = FileManager.Directory.applicationSupport.url.appending(path: "secure_store", directoryHint: .isDirectory),
    ) {
        self.namespace = namespace
        self.keychain = keychain.accessibility(.whenUnlockedThisDeviceOnly, authenticationPolicy: [])
        self.directory = directory
    }

    public func get(key: String) throws -> String? {
        try Self.lock.withLock {
            let url = fileURL(key)
            if FileManager.default.fileExists(atPath: url.path) {
                return try AES.GCM.open(AES.GCM.SealedBox(combined: Data(contentsOf: url)), using: sealingKey(), authenticating: Data(key.utf8)).encodeString()
            }
            guard let value = try keychain.get(namespace + key) else {
                return nil
            }
            try seal(value, key: key)
            return value
        }
    }

    public func set(key: String, value: String) throws {
        try Self.lock.withLock {
            try seal(value, key: key)
        }
    }

    public func remove(key: String) throws {
        try Self.lock.withLock {
            let url = fileURL(key)
            if FileManager.default.fileExists(atPath: url.path) {
                try FileManager.default.removeItem(at: url)
            }
            try keychain.remove(namespace + key)
        }
    }

    private func seal(_ value: String, key: String) throws {
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try FileManager.default.addSkipBackupAttributeToItemAtURL(directory)
        try AES.GCM.seal(Data(value.utf8), using: sealingKey(), authenticating: Data(key.utf8)).combined!.write(to: fileURL(key), options: .atomic)
        try keychain.remove(namespace + key)
    }

    private func sealingKey() throws -> SymmetricKey {
        if let key = try keychain.getData(Self.sealingKey) {
            return SymmetricKey(data: key)
        }
        let key = SymmetricKey(size: .bits256)
        try keychain.set(key.withUnsafeBytes { Data($0) }, key: Self.sealingKey)
        return key
    }

    private func fileURL(_ key: String) -> URL {
        directory.appending(path: Data(SHA256.hash(data: Data(key.utf8))).hex)
    }
}
