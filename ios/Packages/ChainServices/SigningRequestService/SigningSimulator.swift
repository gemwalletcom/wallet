// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.Chain
import enum Gemstone.SignDigestType
import struct Gemstone.SignMessage
import class Gemstone.WalletConnectSimulationClient
import enum Gemstone.SignableTransactionType
import NativeProviderService
import Primitives
import GemstonePrimitives

public protocol SigningSimulatable: Sendable {
    func simulateSignMessage(chain: Gemstone.Chain, signType: SignDigestType, data: String, sessionDomain: String) async throws -> SimulationResult
    func simulateSendTransaction(chain: Gemstone.Chain, transactionType: SignableTransactionType, data: String) async throws -> SimulationResult
}

public extension SigningSimulatable {
    func simulateSignMessage(message: SignMessage, sessionDomain: String) async throws -> SimulationResult {
        try await simulateSignMessage(
            chain: message.chain,
            signType: message.signType,
            data: String(decoding: message.data, as: UTF8.self),
            sessionDomain: sessionDomain,
        )
    }
}

public final class SigningSimulator: SigningSimulatable, Sendable {
    private let client: WalletConnectSimulationClient

    public init(nodeProvider: any NodeURLFetchable, requestInterceptor: any RequestInterceptable = EmptyRequestInterceptor()) {
        client = WalletConnectSimulationClient(provider: NativeProvider(nodeProvider: nodeProvider, requestInterceptor: requestInterceptor))
    }

    public func simulateSignMessage(chain: Gemstone.Chain, signType: SignDigestType, data: String, sessionDomain: String) async throws -> SimulationResult {
        try await client.simulateSignMessage(chain: chain, signType: signType, data: data, sessionDomain: sessionDomain).map()
    }

    public func simulateSendTransaction(chain: Gemstone.Chain, transactionType: SignableTransactionType, data: String) async throws -> SimulationResult {
        try await client.simulateSendTransaction(chain: chain, transactionType: transactionType, data: data).map()
    }
}
