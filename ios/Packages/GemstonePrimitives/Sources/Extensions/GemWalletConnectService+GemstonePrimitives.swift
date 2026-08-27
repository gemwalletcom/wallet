// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.Account
import protocol Gemstone.GemWalletConnectServiceProtocol
import enum Gemstone.WalletConnectionVerificationStatus
import Primitives

public struct WalletConnectSessionApproval: Sendable {
    public let chains: [Chain]
    public let accounts: [Primitives.Account]
    public let methods: [String]
    public let events: [String]
}

public extension GemWalletConnectServiceProtocol {
    func metadata(name: String, description: String, url: String, icons: [String]) throws -> Primitives.ApplicationMetadata {
        try Primitives.ApplicationMetadata(applicationMetadata(name: name, description: description, url: url, icons: icons))
    }

    func prepareSessionProposal(
        wallets: [Wallet],
        currentWalletId: WalletId?,
        requiredChainIds: [String],
        optionalChainIds: [String],
        metadata: Primitives.ApplicationMetadata,
        origin: String?,
        validation: WalletConnectionVerificationStatus,
    ) throws -> (proposal: WalletConnectionSessionProposal, verificationStatus: WalletConnectionVerificationStatus) {
        let result = try prepareSessionProposal(
            wallets: wallets.map { try $0.json() },
            currentWalletId: currentWalletId?.id,
            requiredChainIds: requiredChainIds,
            optionalChainIds: optionalChainIds,
            metadata: metadata.json(),
            origin: origin,
            validation: validation,
        )
        return (try WalletConnectionSessionProposal(result.proposal), result.verificationStatus)
    }

    func sessionApproval(wallet: Wallet, supportedChains: [Chain]) throws -> WalletConnectSessionApproval {
        let approval = sessionApproval(wallet: try wallet.json(), supportedChains: supportedChains.map(\.rawValue))
        return try WalletConnectSessionApproval(
            chains: approval.chains.map { try $0.map() },
            accounts: approval.accounts.map { try $0.map() },
            methods: approval.methods,
            events: approval.events,
        )
    }

    func session(topic: String, accounts: [String], expireAt: Date, metadata: Primitives.ApplicationMetadata) throws -> WalletConnectionSession {
        try WalletConnectionSession(session(topic: topic, accounts: accounts, expireAt: Int64(expireAt.timeIntervalSince1970), metadata: metadata.json()))
    }
}

private extension Gemstone.Account {
    func map() throws -> Primitives.Account {
        Primitives.Account(chain: try chain.map(), address: address, derivationPath: derivationPath, extendedPublicKey: extendedPublicKey)
    }
}
