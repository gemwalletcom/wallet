// Copyright (c). Gem Wallet. All rights reserved.

@preconcurrency import ReownWalletKit

final class WCConnectionsInteractor: Sendable {
    var sessionsStream: AsyncStream<[Session]> {
        WalletKit.instance.sessionsPublisher.asAsyncStream()
    }

    var sessionProposalStream: AsyncStream<(proposal: Session.Proposal, context: VerifyContext?)> {
        WalletKit.instance.sessionProposalPublisher.asAsyncStream()
    }

    var sessionRequestStream: AsyncStream<(request: Request, context: VerifyContext?)> {
        WalletKit.instance.sessionRequestPublisher.asAsyncStream()
    }

    var sessionDeleteStream: AsyncStream<(topic: String, code: Int, message: String)> {
        WalletKit.instance.sessionDeletePublisher
            .map { topic, reason in
                (topic: topic, code: reason.code, message: reason.message)
            }
            .eraseToAnyPublisher()
            .asAsyncStream()
    }

    var sessions: [Session] {
        WalletKit.instance.getSessions()
    }
}
