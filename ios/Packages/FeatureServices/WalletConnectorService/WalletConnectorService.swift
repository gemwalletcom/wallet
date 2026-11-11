// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemWalletConnectAuthentication
import protocol Gemstone.GemWalletConnectClientProtocol
import enum Gemstone.GemWalletConnectEvent
import struct Gemstone.GemWalletConnectProposal
import enum Gemstone.GemWalletConnectRejectionReason
import protocol Gemstone.GemWalletConnectServiceProtocol
import struct Gemstone.GemWalletConnectSessionRequest
import protocol Gemstone.GemWalletSessionServiceProtocol
import enum Gemstone.WalletConnectLink
import GemstonePrimitives
import Primitives
import WebSocketClient

public actor WalletConnectorService {
    private let client: any GemWalletConnectClientProtocol
    private let webSocket: any WebSocketConnectable
    private let events: AsyncStream<GemWalletConnectEvent>
    private let walletSessionService: any GemWalletSessionServiceProtocol
    private let interactor: any WalletConnectorInteractable
    private let service: any GemWalletConnectServiceProtocol
    private var eventsTask: Task<Void, Never>?
    private var relay: RelayConnection = .closed

    private enum RelayConnection {
        case closed
        case opening(UUID)
        case open(Task<Void, Never>)
    }

    public init(
        client: any GemWalletConnectClientProtocol,
        webSocket: any WebSocketConnectable,
        events: AsyncStream<GemWalletConnectEvent>,
        walletSessionService: any GemWalletSessionServiceProtocol,
        interactor: any WalletConnectorInteractable,
        service: any GemWalletConnectServiceProtocol,
    ) {
        self.client = client
        self.webSocket = webSocket
        self.events = events
        self.walletSessionService = walletSessionService
        self.interactor = interactor
        self.service = service
    }
}

// MARK: - WalletConnectorServiceable

extension WalletConnectorService: WalletConnectorServiceable {
    public func hasSessions() async throws -> Bool {
        try await service.hasSessions()
    }

    public func setup() async throws {
        guard eventsTask == nil else { return }
        eventsTask = Task { [weak self, events] in
            for await event in events {
                await self?.receive(event)
            }
        }
        try await client.restore()
        try await connect()
    }

    public func connect() async throws {
        guard eventsTask != nil, try client.needsConnection() else { return }
        await startConnection()
    }

    public func disconnect() async {
        if case let .open(task) = relay {
            task.cancel()
        }
        relay = .closed
        await webSocket.disconnect()
    }

    public func pair(uri: String) async throws {
        try await setup()
        await startConnection()
        try await client.pair(uri: uri)
    }

    public func linkOpened(_ link: WalletConnectLink) {
        client.linkOpened(link: link)
    }

    public func disconnect(sessionId: String) async throws {
        try await setup()
        await startConnection()
        try await client.disconnect(topic: sessionId)
    }

    public func updateSessions() async throws {
        guard eventsTask != nil else {
            return try await setup()
        }
        try await client.restore()
    }
}

// MARK: - Private

extension WalletConnectorService {
    private func startConnection() async {
        guard case .closed = relay else { return }
        let attempt = UUID()
        relay = .opening(attempt)
        let stream = await webSocket.connect()
        guard case let .opening(current) = relay, current == attempt else { return }
        relay = .open(Task { [weak self] in
            for await event in stream {
                await self?.receive(event)
            }
        })
    }

    private func receive(_ event: WebSocketEvent) async {
        do {
            switch event {
            case .connected:
                try await client.connected()
            case let .message(data):
                try await client.receive(frame: String(decoding: data, as: UTF8.self))
            case .disconnected:
                break
            }
        } catch {
            debugLog("WalletConnect relay error: \(error)")
        }
    }

    private func receive(_ event: GemWalletConnectEvent) {
        switch event {
        case let .proposal(proposal):
            Task { await approve(proposal) }
        case let .request(request):
            Task { await answer(request) }
        case let .requestExpired(_, requestId):
            Task { await interactor.requestExpired(requestId: requestId) }
        case let .sessionDeleted(topic):
            debugLog("WalletConnect session deleted by peer: \(topic)")
        case let .returnToDapp(name, redirect):
            Task { await interactor.returnToDapp(name: name, redirect: redirect) }
        case let .authentication(authentication):
            Task { await authenticate(authentication) }
        }
    }

    private func approve(_ proposal: GemWalletConnectProposal) async {
        do {
            let prepared = try await service.prepareSessionProposal(
                requiredChainIds: proposal.requiredChainIds,
                optionalChainIds: proposal.optionalChainIds,
                metadata: proposal.metadata,
                origin: proposal.origin,
                validation: proposal.validation,
            )
            let walletId = try await interactor.sessionApproval(payload: WCPairingProposal(
                pairingId: proposal.pairingTopic,
                proposal: prepared.proposal.toPrimitives(),
                verificationStatus: prepared.verificationStatus.toPrimitives(),
            ))
            let wallet = try await walletSessionService.requireWallet(walletId: walletId)
            try await client.approveSession(proposalId: proposal.id, wallet: wallet)
        } catch {
            await reject(proposal, error: error)
        }
    }

    private func reject(_ proposal: GemWalletConnectProposal, error: any Error) async {
        do {
            try await client.rejectSession(proposalId: proposal.id, reason: GemWalletConnectRejectionReason(from: error))
        } catch {
            debugLog("WalletConnect reject proposal error: \(error)")
        }
        await interactor.sessionReject(error: error)
    }

    private func authenticate(_ authentication: GemWalletConnectAuthentication) async {
        do {
            let prepared = try await service.prepareAuthentication(
                chainIds: authentication.chainIds,
                domain: authentication.domain,
                metadata: authentication.metadata,
                origin: authentication.origin,
                validation: authentication.validation,
            )
            try await interactor.authenticationApproval(request: WCAuthenticationRequest(
                id: authentication.id,
                proposal: prepared.proposal.toPrimitives(),
                verificationStatus: prepared.verificationStatus.toPrimitives(),
                message: { [client] wallet in
                    try client.authenticationMessage(authenticationId: authentication.id, wallet: wallet.toGem())
                },
                approve: { [client] wallet, signature in
                    try await client.approveAuthentication(authenticationId: authentication.id, wallet: wallet.toGem(), signature: signature)
                },
            ))
        } catch {
            await reject(authentication, error: error)
        }
    }

    private func reject(_ authentication: GemWalletConnectAuthentication, error: any Error) async {
        do {
            try await client.rejectAuthentication(authenticationId: authentication.id)
        } catch {
            debugLog("WalletConnect reject authentication error: \(error)")
        }
        await interactor.sessionReject(error: error)
    }

    private func answer(_ request: GemWalletConnectSessionRequest) async {
        do {
            if let failure = try await client.answerRequest(topic: request.topic, requestId: request.requestId) {
                await interactor.sessionReject(error: failure)
            }
        } catch {
            debugLog("WalletConnect answer request error: \(error)")
        }
    }
}
