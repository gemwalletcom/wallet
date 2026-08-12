// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

public struct SimulationPayloadModel: Sendable {
    public let chain: Chain
    public let primaryFields: [SimulationPayloadField]
    public let secondaryFields: [SimulationPayloadField]
    public var addressNames: [ChainAddress: AddressName]

    public init(
        chain: Chain,
        primaryFields: [SimulationPayloadField],
        secondaryFields: [SimulationPayloadField],
        addressNames: [ChainAddress: AddressName] = [:],
    ) {
        self.chain = chain
        self.primaryFields = primaryFields
        self.secondaryFields = secondaryFields
        self.addressNames = addressNames
    }

    public var hasFields: Bool { primaryFields.isNotEmpty || secondaryFields.isNotEmpty }
    public var hasDetails: Bool { secondaryFields.isNotEmpty }

    public var addressRequests: [ChainAddress] {
        (primaryFields + secondaryFields).compactMap {
            guard $0.fieldType == .address else {
                return nil
            }
            return ChainAddress(chain: chain, address: $0.value)
        }
    }

    public func fieldViewModel(for field: SimulationPayloadField) -> SimulationPayloadFieldViewModel {
        SimulationPayloadFieldViewModel(
            field: field,
            chain: chain,
            addressName: addressNames[ChainAddress(chain: chain, address: field.value)],
        )
    }

    public func contextMenuItems(
        for field: SimulationPayloadField,
        explorerLink: (String) -> BlockExplorerLink,
        onOpenURL: @escaping (URL) -> Void,
    ) -> [ContextMenuItemType] {
        var items = fieldViewModel(for: field).contextMenuItems
        guard field.fieldType == .address else {
            return items
        }

        let link = explorerLink(field.value)
        items.append(.url(title: Localized.Transaction.viewOn(link.name), onOpen: {
            if let url = URL(string: link.link) {
                onOpenURL(url)
            }
        }))
        return items
    }
}
