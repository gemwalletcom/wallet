// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAddressDetails
import protocol Gemstone.GemAddressDetailsServiceProtocol
import enum Gemstone.GemListRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

@MainActor
@Observable
public final class AddressDetailsSceneViewModel {
    private let service: any GemAddressDetailsServiceProtocol
    private var details: GemAddressDetails

    public init(
        chainAddress: ChainAddress,
        service: any GemAddressDetailsServiceProtocol,
    ) {
        self.service = service
        details = service.details(chain: chainAddress.chain.rawValue, address: chainAddress.address)
    }

    var title: String {
        Localized.Common.address
    }
}

// MARK: - ListSectionProvideable

extension AddressDetailsSceneViewModel: ListSectionProvideable {
    public var sections: [ListSection<GemListSectionRow>] {
        details.sections().listSections
    }
}

// MARK: - Actions

extension AddressDetailsSceneViewModel {
    func refresh() async {
        details = await service.refresh(details: details)
    }
}
