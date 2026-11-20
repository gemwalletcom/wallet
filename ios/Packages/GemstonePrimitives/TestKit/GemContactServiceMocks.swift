// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
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
    public var connectionGroupsValue: [GemGroup] = []
    public var connectionRowValue = GemConnectionRow(title: "", host: nil, initial: "WC", iconUrl: nil)
    public var connectionDetailRows: [GemListRow] = []
    public var hasSessionsValue = false

    public private(set) var viewedConnections: [Gemstone.WalletConnection] = []

    public init() {}

    public func connectionDetails(connection: Gemstone.WalletConnection) -> GemConnectionDetails {
        GemConnectionDetails(
            connection: GemConnection(connection: connection, row: connectionRowValue),
            rows: connectionDetailRows,
        )
    }

    public func connectionsView(connections: [Gemstone.WalletConnection]) -> GemConnectionsView {
        viewedConnections = connections
        return GemConnectionsView(
            list: GemGroupList(
                groups: connectionGroupsValue,
                phase: connectionGroupsValue.isEmpty ? .empty(state: emptyState(kind: .walletConnect)) : .rows,
            ),
            docsUrl: "https://docs.gemwallet.com/guides/walletconnect/",
        )
    }

    public func hasSessions() async throws -> Bool {
        hasSessionsValue
    }

    public func prepareAuthentication(
        chainIds _: [String],
        domain _: String,
        metadata: Gemstone.ApplicationMetadata,
        origin: String?,
        validation: Gemstone.WalletConnectionVerificationStatus,
    ) async throws -> GemSessionProposal {
        try await prepareSessionProposal(requiredChainIds: [], optionalChainIds: [], metadata: metadata, origin: origin, validation: validation)
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
}

public final class GemSupportServiceMock: GemSupportServiceProtocol, @unchecked Sendable {
    public var imageFilePath = "/tmp/support.png"
    public var sendError: Error?
    public var syncError: Error?
    public var imageFileError: Error?
    public var syncFromTimestampValue: UInt64 = 0

    public private(set) var syncCursorMessageIds: [String] = []
    public private(set) var syncedTimestamps: [UInt64] = []
    public private(set) var sentTexts: [String] = []
    public private(set) var retriedMessageIds: [String] = []
    public private(set) var requestedImageUrls: [String] = []
    public private(set) var recoveredInterrupted = 0
    public var pushState: GemPushState?
    public private(set) var enableNotificationsCalls = 0

    public init() {}

    public func enableNotifications() async -> GemPushState? {
        enableNotificationsCalls += 1
        return pushState
    }

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
        syncCursorMessageIds.append(contentsOf: messages.map(\.id))
        return syncFromTimestampValue
    }

    public func refresh(fromTimestamp: UInt64) async -> GemLoadState {
        syncedTimestamps.append(fromTimestamp)
        guard let syncError else {
            return .data
        }
        return .error(error: .Api(msg: syncError.localizedDescription))
    }
}
