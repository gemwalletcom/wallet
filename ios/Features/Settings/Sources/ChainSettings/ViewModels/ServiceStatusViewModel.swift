// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemListSection
import protocol Gemstone.GemServiceStatusProtocol
import Localization
import PrimitivesComponents

@Observable
@MainActor
public final class ServiceStatusViewModel {
    private let service: any GemServiceStatusProtocol
    private var items: [GemListSection]

    public init(service: any GemServiceStatusProtocol) {
        self.service = service
        items = service.sections()
    }

    var title: String { Localized.Transaction.status }
}

extension ServiceStatusViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemListSectionRow>] { items.listSections }
}

// MARK: - Actions

extension ServiceStatusViewModel {
    func load() async {
        items = service.sections()
        let result = await service.load()
        guard !Task.isCancelled else { return }
        items = result
    }
}
