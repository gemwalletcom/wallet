// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemLatencyStatus
import struct Gemstone.GemListSection
import protocol Gemstone.GemServiceStatusProtocol
import struct Gemstone.GemServiceStatusSession
import enum Gemstone.GemServiceStatusTarget
import Localization
import PrimitivesComponents

@Observable
@MainActor
public final class ServiceStatusViewModel {
    private let service: any GemServiceStatusProtocol
    private var session: GemServiceStatusSession

    public init(service: any GemServiceStatusProtocol) {
        self.service = service
        session = service.newSession()
    }

    var title: String { Localized.Transaction.status }
}

public extension ServiceStatusViewModel {
    var sections: [GemListSection] { session.sections() }
}

// MARK: - Actions

extension ServiceStatusViewModel {
    func load() async {
        session = service.newSession()
        let service = service
        await withTaskGroup(of: (GemServiceStatusTarget, GemLatencyStatus).self) { group in
            for target in session.targets() {
                group.addTask {
                    await (target, service.status(target: target))
                }
            }

            for await (target, status) in group {
                guard !Task.isCancelled else { return }
                session = session.onStatus(target: target, status: status)
            }
        }
    }
}
