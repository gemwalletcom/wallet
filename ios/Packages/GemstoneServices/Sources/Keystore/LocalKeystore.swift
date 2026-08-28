public import Gemstone
import Foundation
import GemstonePrimitives
import Primitives

public final class LocalKeystore: Keystore, @unchecked Sendable {
    public let gemKeystore: GemKeystore
    private let keystoreURL: URL
    private let keystorePassword: KeystorePassword
    private let queue = DispatchQueue(label: "com.gemwallet.keystore", qos: .userInitiated)

    public init(
        directory: String = "keystore",
        keystorePassword: KeystorePassword = LocalKeystorePassword(),
    ) {
        do {
            // migrate keystore from documents directory to application support directory
            // TODO: delete in 2026
            let fileMigrator = FileMigrator()
            let keystoreURL = try fileMigrator.migrate(
                name: directory,
                fromDirectory: .documentDirectory,
                toDirectory: .applicationSupportDirectory,
                isDirectory: true,
            )
            self.keystoreURL = keystoreURL
            gemKeystore = try GemKeystore(baseDir: keystoreURL.path)
        } catch {
            fatalError("keystore initialization error: \(error)")
        }

        self.keystorePassword = keystorePassword
    }

    public func keystorePassword(createIfMissing: Bool) throws -> Data {
        let password = try keystorePassword.getPassword()
        if password.isNotEmpty {
            return try password.v4KeystorePasswordBytes()
        }
        guard createIfMissing, try !gemKeystore.hasStoredWallets() else {
            throw AnyError("Couldn't access this wallet's keys on this device. If you have your recovery phrase, remove this wallet and import it again to restore access.")
        }
        let newPassword = try SecureRandom.generateKey(length: 32).hex
        try keystorePassword.setPassword(newPassword, authentication: .none)
        return try newPassword.v4KeystorePasswordBytes()
    }

    public func migrateV3Keystores(for wallets: [Primitives.Wallet]) async throws -> [KeystoreMigrationFailure] {
        let pendingWallets = pendingV3Migrations(for: wallets)
        guard pendingWallets.isNotEmpty else {
            return []
        }
        let password = try await getPassword()
        guard !password.isEmpty else {
            return pendingWallets.map {
                KeystoreMigrationFailure(walletId: $0.wallet.id, error: AnyError("keystore password is missing"))
            }
        }

        var failures: [KeystoreMigrationFailure] = []
        for pending in pendingWallets {
            do {
                try await migrateV3Keystore(wallet: pending.wallet, v3URL: pending.v3URL, password: password)
            } catch {
                failures.append(KeystoreMigrationFailure(walletId: pending.wallet.id, error: error))
            }
        }
        return failures
    }

    public func deleteKey(for wallet: Primitives.Wallet) async throws {
        switch wallet.type {
        case .view: break
        case .multicoin, .single, .privateKey:
            try await queue.asyncTask { [gemKeystore, keystoreURL] in
                _ = try gemKeystore.delete(keystoreId: wallet.keystoreId)
                if let legacyURL = Self.findV3File(in: keystoreURL, matching: wallet.legacyV3Id) {
                    try FileManager.default.removeItem(at: legacyURL)
                }
            }
        }
    }

    public func sign(wallet: Primitives.Wallet, input: GemSignerInput) async throws -> [GemSignedTransaction] {
        let password = try await getPassword()
        let keystoreId = wallet.keystoreId
        let chain = try Primitives.Asset(GemTransferService().asset(inputType: input.input.inputType)).id.chain.rawValue
        return try await queue.asyncTask { [gemKeystore] in
            try withV4Password(password) { passwordBytes in
                try gemKeystore.sign(keystoreId: keystoreId, chain: chain, input: input, password: passwordBytes)
            }
        }
    }

    public func signMessage(signer: MessageSigner, wallet: Primitives.Wallet) async throws -> String {
        let password = try await getPassword()
        let keystoreId = wallet.keystoreId
        return try await queue.asyncTask { [gemKeystore] in
            try withV4Password(password) { passwordBytes in
                try signer.signWithKeystore(keystore: gemKeystore, keystoreId: keystoreId, password: passwordBytes)
            }
        }
    }

    public func getPrivateKeyEncoded(wallet: Primitives.Wallet, chain: Primitives.Chain) async throws -> String {
        let password = try await getPassword()
        return try await queue.asyncTask { [gemKeystore] in
            try withV4Password(password) { passwordBytes in
                try gemKeystore.exportPrivateKey(
                    keystoreId: wallet.keystoreId,
                    chain: chain.rawValue,
                    password: passwordBytes,
                )
            }
        }
    }

    public func getMnemonic(wallet: Primitives.Wallet) async throws -> [String] {
        let password = try await getPassword()
        return try await queue.asyncTask { [gemKeystore] in
            try withV4Password(password) { passwordBytes in
                try gemKeystore.exportRecoveryPhrase(
                    keystoreId: wallet.keystoreId,
                    password: passwordBytes,
                )
            }
        }
    }

    public func getPasswordAuthentication() throws -> KeystoreAuthentication {
        try keystorePassword.getAuthentication()
    }

    public func destroy() throws {
        guard FileManager.default.fileExists(atPath: keystoreURL.path) else {
            return
        }
        try FileManager.default.removeItem(at: keystoreURL)
    }

    @MainActor
    func getPassword() throws -> String {
        try keystorePassword.getPassword()
    }

    private func pendingV3Migrations(for wallets: [Primitives.Wallet]) -> [(wallet: Primitives.Wallet, v3URL: URL)] {
        wallets.compactMap { wallet in
            switch wallet.type {
            case .view:
                return nil
            case .multicoin, .single, .privateKey:
                guard let v3URL = Self.findV3File(in: keystoreURL, matching: wallet.legacyV3Id) else {
                    return nil
                }
                return (wallet, v3URL)
            }
        }
    }

    private func migrateV3Keystore(wallet: Primitives.Wallet, v3URL: URL, password: String) async throws {
        try await queue.asyncTask { [gemKeystore] in
            var v3Password = password.v3PasswordBytes()
            var newPassword = try password.v4KeystorePasswordBytes()
            defer {
                v3Password.zeroize()
                newPassword.zeroize()
            }
            _ = try gemKeystore.migrateV3(
                v3Path: v3URL.path,
                v3Password: v3Password,
                newPassword: newPassword,
                walletId: wallet.id.id,
            )
        }
    }

    private static func findV3File(in directory: URL, matching keystoreId: String) -> URL? {
        let target = keystoreId.lowercased()
        let fileManager = FileManager.default
        guard let contents = try? fileManager.contentsOfDirectory(
            at: directory,
            includingPropertiesForKeys: [.isDirectoryKey],
        ) else {
            return nil
        }
        for url in contents {
            let isDirectory = (try? url.resourceValues(forKeys: [.isDirectoryKey]))?.isDirectory ?? false
            if isDirectory { continue }
            let name = url.lastPathComponent.lowercased()
            if name == target || name.hasSuffix(target) {
                return url
            }
            if let data = try? Data(contentsOf: url),
               let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
               let fileId = (json["id"] as? String)?.lowercased(),
               fileId == target
            {
                return url
            }
        }
        return nil
    }
}

func withV4Password<T>(
    _ password: String,
    _ operation: (Data) throws -> T,
) throws -> T {
    guard password.isNotEmpty else {
        throw AnyError("Couldn't access this wallet's keys on this device. If you have your recovery phrase, remove this wallet and import it again to restore access.")
    }
    var passwordBytes = try password.v4KeystorePasswordBytes()
    defer { passwordBytes.zeroize() }
    return try operation(passwordBytes)
}
