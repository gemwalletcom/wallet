// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config
import enum Gemstone.GemServiceError
import protocol Gemstone.GemWalletConnectSigner
import struct Gemstone.GemWalletConnectSignRequest
import GemstonePrimitives
import GemstoneServices
import Primitives
import WalletConnectorService

public final class WalletConnectorSigner: WalletConnectorSignable, GemWalletConnectSigner {
    private let walletConnectorInteractor: any WalletConnectorInteractable
    private let walletSessionService: any WalletSessionManageable

    public init(
        walletSessionService: any WalletSessionManageable,
        walletConnectorInteractor: any WalletConnectorInteractable,
    ) {
        self.walletConnectorInteractor = walletConnectorInteractor
        self.walletSessionService = walletSessionService
    }

    public var allChains: [Primitives.Chain] {
        Config.shared.getWalletConnectConfig().chains.compactMap { Primitives.Chain(rawValue: $0) }
    }

    public func getCurrentWallet() throws -> Wallet {
        try walletSessionService.getCurrentWallet()
    }

    public func getWallet(id: WalletId) throws -> Wallet {
        try walletSessionService.getWallet(walletId: id)
    }

    public func getWallets() throws -> [Wallet] {
        try walletSessionService.getWallets()
    }

    public func sessionApproval(payload: WCPairingProposal) async throws -> WalletId {
        try await walletConnectorInteractor.sessionApproval(payload: payload)
    }

    public func sessionReject(error: any Error) async {
        await walletConnectorInteractor.sessionReject(error: error)
    }

    public func sign(request: GemWalletConnectSignRequest) async throws -> String {
        let chain = try request.chain.map()
        let session = try WalletConnectionSession(request.session)
        let wallet = try Wallet(request.wallet)
        let simulation = try SimulationResult(request.simulation)

        switch request.payload {
        case let .message(message):
            let payload = SignMessagePayload(chain: chain, session: session, wallet: wallet, message: message, simulation: simulation)
            return try await interact { try await walletConnectorInteractor.signMessage(payload: payload) }
        case let .transaction(transfer, action):
            let data = try WCTransferData(transferData: TransferData(transfer), wallet: wallet, simulation: simulation)
            return try await interact {
                switch action {
                case .sign: try await walletConnectorInteractor.signTransaction(transferData: data)
                case .send: try await walletConnectorInteractor.sendTransaction(transferData: data)
                }
            }
        }
    }

    private func interact(_ action: () async throws -> String) async throws -> String {
        do {
            return try await action()
        } catch ConnectionsError.userCancelled {
            throw GemServiceError.Cancelled
        }
    }
}
