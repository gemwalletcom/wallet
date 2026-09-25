// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import enum Gemstone.GemNameInputStep
import struct Gemstone.GemPriceAlertSession
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension GemContactService {
    static func mock() -> GemContactService {
        GemContactService(
            store: GemContactStoreMock(),
            names: .mock(),
            files: GemFileStoreMock(),
        )
    }
}

public final class GemWalletConnectServiceMock: GemWalletConnectServiceProtocol, @unchecked Sendable {
    public var connectionSectionsValue: [GemConnectionSection] = []
    public var connectionRowValue = GemConnectionRow(title: "", host: nil, initial: "WC", iconUrl: nil)
    public var connectionDetailRows: [GemListRow] = []
    public var hasSessionsValue = false
    public var signatureResult: Result<String, Error> = .success("0x")

    public private(set) var addedConnections: [Gemstone.WalletConnection] = []
    public private(set) var deletedSessionIds: [String] = []
    public private(set) var updatedSessions: [[Gemstone.WalletConnectionSession]] = []
    public private(set) var processedRequests: [GemWalletConnectSessionRequest] = []
    public private(set) var seenMessageIds: [String] = []

    public init() {}

    public func addConnection(connection: Gemstone.WalletConnection) async throws {
        addedConnections.append(connection)
    }

    public func applicationMetadata(name: String, description: String, url: String, icons: [String]) -> Gemstone.ApplicationMetadata {
        Gemstone.ApplicationMetadata(name: name, description: description, url: url, icon: icons.first ?? "", source: .walletConnect)
    }

    public func authenticationAccounts(chainIds _: [String], wallet _: Gemstone.Wallet) -> [GemWalletConnectAuthAccount] {
        []
    }

    public func authenticationChainIds(chainIds: [String]) -> [String] {
        chainIds
    }

    public func authenticationMethods() -> [String] {
        []
    }

    public func configSessionProperties(properties: [String: String], caip2Chains _: [String], accounts _: [Gemstone.Account]) -> [String: String] {
        properties
    }

    public func connectionDetails(connection: Gemstone.WalletConnection) -> GemConnectionDetails {
        GemConnectionDetails(
            connection: GemConnection(connection: connection, row: connectionRowValue),
            rows: connectionDetailRows,
        )
    }

    public func connectionsView(connections _: [Gemstone.WalletConnection]) -> GemConnectionsView {
        GemConnectionsView(sections: connectionSectionsValue, docsUrl: "https://docs.gemwallet.com/guides/walletconnect/")
    }

    public func deleteSession(sessionId: String) async throws {
        deletedSessionIds.append(sessionId)
    }

    public func hasSessions() async throws -> Bool {
        hasSessionsValue
    }

    public func prepareSessionProposal(
        requiredChainIds _: [String],
        optionalChainIds _: [String],
        metadata: Gemstone.ApplicationMetadata,
        origin _: String?,
        validation: Gemstone.WalletConnectionVerificationStatus,
    ) async throws -> GemSessionProposal {
        GemSessionProposal(
            proposal: Gemstone.WalletConnectionSessionProposal(defaultWallet: Primitives.Wallet.mock().toGem(), wallets: [], metadata: metadata),
            verificationStatus: validation,
        )
    }

    public func requestOutcome(request: GemWalletConnectSessionRequest) async -> GemWalletConnectOutcome {
        processedRequests.append(request)
        return GemWalletConnectOutcome(response: nil, failure: nil)
    }

    public func session(topic: String, accounts _: [String], expireAt: Int64, metadata: Gemstone.ApplicationMetadata) throws -> Gemstone.WalletConnectionSession {
        Gemstone.WalletConnectionSession(
            id: topic,
            sessionId: topic,
            state: .active,
            chains: [],
            createdAt: Date(timeIntervalSince1970: 0),
            expireAt: Date(timeIntervalSince1970: TimeInterval(expireAt)),
            metadata: metadata,
        )
    }

    public func sessionRejection(reason: GemWalletConnectRejectionReason) -> GemWalletConnectRejection {
        GemWalletConnectRejection(reason: reason, code: 4001, message: "Rejected", deletesSession: true)
    }

    public func sessionApproval(wallet _: Gemstone.Wallet) -> GemSessionApproval {
        GemSessionApproval(chains: [], accounts: [], methods: [], events: [])
    }

    public func shouldProcessProposal(proposerPublicKey: String) -> Bool {
        let messageId = "proposal-\(proposerPublicKey)"
        let seen = seenMessageIds.contains(messageId)
        seenMessageIds.append(messageId)
        return !seen
    }

    public func signMessage(walletId _: Gemstone.WalletId, message _: Gemstone.SignMessage) async throws -> String {
        try signatureResult.get()
    }

    public func updateSessions(sessions: [Gemstone.WalletConnectionSession]) async throws {
        updatedSessions.append(sessions)
    }
}

public final class GemSupportServiceMock: GemSupportServiceProtocol, @unchecked Sendable {
    public var imageFilePath = "/tmp/support.png"
    public var sendError: Error?
    public var syncError: Error?
    public var imageFileError: Error?

    public private(set) var syncedTimestamps: [UInt64] = []
    public private(set) var sentTexts: [String] = []
    public private(set) var retriedMessageIds: [String] = []
    public private(set) var requestedImageUrls: [String] = []
    public private(set) var recoveredInterrupted = 0

    public init() {}

    public func imageFile(url: String) async throws -> String {
        requestedImageUrls.append(url)
        if let imageFileError {
            throw imageFileError
        }
        return imageFilePath
    }

    public func recoverInterruptedMessages() async throws {
        recoveredInterrupted += 1
    }

    public func retryMessage(message: Gemstone.SupportMessage) async throws {
        retriedMessageIds.append(message.id)
        if let sendError {
            throw sendError
        }
    }

    public func sendImage(image _: Data) async throws {
        if let sendError {
            throw sendError
        }
    }

    public func sendText(content: String) async throws {
        sentTexts.append(content)
        if let sendError {
            throw sendError
        }
    }

    public func syncFromTimestamp(messages: [Gemstone.SupportMessage]) -> UInt64 {
        messages.last {
            if case .agent = $0.sender {
                true
            } else {
                false
            }
        }
        .map { UInt64(max($0.createdAt.timeIntervalSince1970, 0)) } ?? 0
    }

    public func refresh(fromTimestamp: UInt64, hasMessages: Bool) async -> GemLoadState {
        syncedTimestamps.append(fromTimestamp)
        guard let syncError else {
            return .data
        }
        return hasMessages ? .data : .error(error: .Api(msg: syncError.localizedDescription))
    }
}
