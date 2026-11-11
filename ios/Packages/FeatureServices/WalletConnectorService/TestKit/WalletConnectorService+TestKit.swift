// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletConnectClientProtocol
import class Gemstone.GemWalletSessionService
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import WalletConnectorService
import WebSocketClient

public extension WalletConnectorService {
    static func mock(
        client: any GemWalletConnectClientProtocol = GemWalletConnectClientMock(),
        webSocket: any WebSocketConnectable,
    ) -> WalletConnectorService {
        WalletConnectorService(
            client: client,
            webSocket: webSocket,
            events: AsyncStream { $0.finish() },
            walletSessionService: GemWalletSessionService.mock(),
            interactor: WalletConnectorInteractorMock(),
            service: GemWalletConnectServiceMock(),
        )
    }
}

public struct WalletConnectorInteractorMock: WalletConnectorInteractable {
    public init() {}

    public func sessionReject(error _: any Error) async {}

    public func sessionApproval(payload _: WCPairingProposal) async throws -> WalletId {
        throw AnyError("rejected")
    }

    public func requestExpired(requestId _: String) async {}

    public func returnToDapp(name _: String, redirect _: String?) async {}
}
