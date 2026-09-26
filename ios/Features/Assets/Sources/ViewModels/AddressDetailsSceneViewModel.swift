// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAddressDetails
import protocol Gemstone.GemAddressDetailsServiceProtocol
import struct Gemstone.GemCopy
import enum Gemstone.GemListRow
import struct Gemstone.GemListSection
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store

@MainActor
@Observable
public final class AddressDetailsSceneViewModel {
    private let service: any GemAddressDetailsServiceProtocol
    private var details: GemAddressDetails

    public let addressNameQuery: ObservableQuery<AddressNameQuery>

    var copyToast: GemCopy?

    public init(
        chainAddress: ChainAddress,
        service: any GemAddressDetailsServiceProtocol,
    ) {
        self.service = service
        addressNameQuery = ObservableQuery(AddressNameQuery(chain: chainAddress.chain, address: chainAddress.address), initialValue: nil)
        details = service.details(chain: chainAddress.chain.rawValue, address: chainAddress.address)
    }

    var title: String {
        Localized.Common.address
    }
}

public extension AddressDetailsSceneViewModel {
    var sections: [GemListSection] {
        details.sections(addressName: addressNameQuery.value?.toGem())
    }
}

// MARK: - Actions

extension AddressDetailsSceneViewModel {
    func refresh() async {
        details = await service.refresh(details: details)
    }

    func onCopy(_ copy: GemCopy) {
        copyToast = copy
    }
}
