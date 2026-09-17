// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import struct Gemstone.GemPriceAlertSession
import enum Gemstone.GemNameInputStep

public extension GemContactService {
    static func mock() -> GemContactService {
        GemContactService(
            store: GemContactStoreMock(),
            addressStore: GemAddressStoreMock(),
            files: GemFileStoreMock(),
        )
    }
}

public final class GemWalletConnectServiceMock: GemWalletConnectServiceProtocol, @unchecked Sendable {
    public var connectionSectionsValue: [GemConnectionSection] = []
    public var connectionRowValue = GemConnectionRow(title: "", host: nil, initial: nil, iconUrl: nil)
    public var connectionDetailRows: [GemConnectionDetailRow] = []
    public var originRejected = false
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

    public func authenticationAccounts(chainIds _: [String], wallet _: Gemstone.Wallet) -> [GemWalletConnectAuthAccount] { [] }

    public func authenticationChainIds(chainIds: [String]) -> [String] { chainIds }

    public func authenticationMethods() -> [String] { [] }

    public func configSessionProperties(properties: [String: String], caip2Chains _: [String], accounts _: [Gemstone.Account]) -> [String: String] {
        properties
    }

    public func connectionDetails(connection: Gemstone.WalletConnection) -> GemConnectionDetails {
        GemConnectionDetails(
            connection: GemConnection(connection: connection, row: connectionRowValue),
            rows: connectionDetailRows,
            wallet: "",
            date: Date(timeIntervalSince1970: 0),
        )
    }

    public func connectionRow(metadata _: Gemstone.ApplicationMetadata) -> GemConnectionRow { connectionRowValue }

    public func connectionSections(connections _: [Gemstone.WalletConnection]) -> [GemConnectionSection] { connectionSectionsValue }

    public func deleteSession(sessionId: String) async throws {
        deletedSessionIds.append(sessionId)
    }

    public func hasSessions() async throws -> Bool { hasSessionsValue }

    public func isOriginRejected(metadataUrl _: String, origin _: String?, validation _: Gemstone.WalletConnectionVerificationStatus) -> Bool {
        originRejected
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

    public func processRequest(request: GemWalletConnectSessionRequest) async -> GemWalletConnectOutcome {
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

    public func shouldProcessMessage(messageId: String) -> Bool {
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

    public func userRejectedError() -> GemWalletConnectRpcError {
        GemWalletConnectRpcError(code: 4001, message: "User rejected")
    }
}

public final class GemSupportServiceMock: GemSupportServiceProtocol, @unchecked Sendable {
    public var imageFilePath = "/tmp/support.png"
    public var sendError: Error?
    public var syncError: Error?
    public var imageFileError: Error?

    public private(set) var syncedTimestamps: [UInt64] = []
    public private(set) var sentTexts: [String] = []
    public private(set) var sentImages: [String] = []
    public private(set) var retriedMessageIds: [String] = []
    public private(set) var requestedImageUrls: [String] = []

    public init() {}

    public func imageFile(url: String) async throws -> String {
        requestedImageUrls.append(url)
        if let imageFileError { throw imageFileError }
        return imageFilePath
    }

    public func retryMessage(message: Gemstone.SupportMessage) async throws {
        retriedMessageIds.append(message.id)
        if let sendError { throw sendError }
    }

    public func sendImage(image _: Data, fileName: String, mimeType _: String) async throws {
        sentImages.append(fileName)
        if let sendError { throw sendError }
    }

    public func sendText(content: String) async throws {
        sentTexts.append(content)
        if let sendError { throw sendError }
    }

    public func syncFromTimestamp(messages: [Gemstone.SupportMessage]) -> UInt64 {
        messages.last { if case .agent = $0.sender { return true } else { return false } }
            .map { UInt64(max($0.createdAt.timeIntervalSince1970, 0)) } ?? 0
    }

    public func syncMessages(fromTimestamp: UInt64) async throws {
        syncedTimestamps.append(fromTimestamp)
        if let syncError { throw syncError }
    }
}
