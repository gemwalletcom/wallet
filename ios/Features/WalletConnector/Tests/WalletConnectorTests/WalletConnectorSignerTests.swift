// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SignMessage
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import WalletConnector
import WalletConnectorService
import WalletConnectSign
import GemstoneServices
import GemstoneServicesTestKit

struct WalletConnectorSignerTests {
    @Test
    func validateChainPresent() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionsStore(db: db)

        let wallet = Wallet.mock(id: .multicoin(address: "0x1"), accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(wallet)

        let signer = WalletConnectorSigner.mock(
            connectionsStore: connectionsStore,
            walletSessionService: WalletSessionService.mock(store: walletStore),
        )

        let sessionId = "session-chain-test"
        try signer.addConnection(connection: WalletConnection(
            session: .mock(id: sessionId, sessionId: sessionId, chains: [.ethereum]),
            wallet: wallet,
        ))

        let message = SignMessage(chain: "ethereum", signType: .eip191, data: Data())
        await #expect(throws: WalletConnectorServiceError.unresolvedChainId(Chain.polygon.rawValue)) {
            try await signer.signMessage(sessionId: sessionId, chain: Chain.polygon.rawValue, message: message, simulation: SimulationResult.mock().json())
        }
    }

    @Test
    func validateChainEmptyChains() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionsStore(db: db)

        let wallet = Wallet.mock(id: .multicoin(address: "0x1"), accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(wallet)

        let signer = WalletConnectorSigner.mock(
            connectionsStore: connectionsStore,
            walletSessionService: WalletSessionService.mock(store: walletStore),
        )

        let sessionId = "session-empty-chains"
        try signer.addConnection(connection: WalletConnection(
            session: .mock(id: sessionId, sessionId: sessionId, chains: []),
            wallet: wallet,
        ))

        let message = SignMessage(chain: "ethereum", signType: .eip191, data: Data())
        await #expect(throws: WalletConnectorServiceError.unresolvedChainId(Chain.ethereum.rawValue)) {
            try await signer.signMessage(sessionId: sessionId, chain: Chain.ethereum.rawValue, message: message, simulation: SimulationResult.mock().json())
        }
    }

    @Test
    func sessionBindsToConnectedWallet() throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore(db: db)
        let connectionsStore = ConnectionsStore(db: db)

        let walletA = Wallet.mock(id: .multicoin(address: "0xa"), name: "Wallet A", accounts: [.mock(chain: .ethereum)])
        let walletB = Wallet.mock(id: .multicoin(address: "0xb"), name: "Wallet B", accounts: [.mock(chain: .ethereum)])

        try walletStore.addWallet(walletA)
        try walletStore.addWallet(walletB)

        let signer = WalletConnectorSigner.mock(
            connectionsStore: connectionsStore,
            walletSessionService: WalletSessionService.mock(store: walletStore),
        )

        let sessionAId = "session-for-wallet-a"
        let sessionBId = "session-for-wallet-b"

        try signer.addConnection(connection: WalletConnection(
            session: .mock(id: sessionAId, sessionId: sessionAId, chains: [.ethereum]),
            wallet: walletA,
        ))
        try signer.addConnection(connection: WalletConnection(
            session: .mock(id: sessionBId, sessionId: sessionBId, chains: [.ethereum]),
            wallet: walletB,
        ))

        let connectionA = try connectionsStore.getConnection(id: sessionAId)
        let connectionB = try connectionsStore.getConnection(id: sessionBId)

        #expect(connectionA.wallet.id == walletA.id)
        #expect(connectionB.wallet.id == walletB.id)
    }
}

extension WalletConnectorSigner {
    static func mock(
        connectionsStore: ConnectionsStore = .mock(),
        walletSessionService: any WalletSessionManageable = WalletSessionService.mock(
            store: .mock(),
            preferences: .mock(),
        ),
    ) -> WalletConnectorSigner {
        WalletConnectorSigner(
            connectionsStore: connectionsStore,
            walletSessionService: walletSessionService,
            walletConnectorInteractor: WalletConnectorManager(presenter: WalletConnectorPresenter()),
        )
    }
}
