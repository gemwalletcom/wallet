// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import class Gemstone.GemAddressService
import GemstonePrimitives
import Localization
import Primitives
import struct Gemstone.SimulationPayloadField

public struct SimulationPayloadFieldViewModel: Identifiable {
    public let field: SimulationPayloadField
    public let chain: Chain
    public let addressName: AddressName?
    public let relativeDateFormatter: RelativeDateFormatter
    public let explorerItem: ContextMenuItemType?

    public init(
        field: SimulationPayloadField,
        chain: Chain,
        addressName: AddressName? = nil,
        relativeDateFormatter: RelativeDateFormatter = RelativeDateFormatter(),
        explorerItem: ContextMenuItemType? = nil,
    ) {
        self.field = field
        self.chain = chain
        self.addressName = addressName
        self.relativeDateFormatter = relativeDateFormatter
        self.explorerItem = explorerItem
    }

    public var id: SimulationPayloadField {
        field
    }

    public var title: String {
        field.kind.title ?? field.label ?? ""
    }

    public var subtitle: String {
        switch field.fieldType {
        case .address:
            let address = GemAddressService.shared.format(address: field.value, chain: chain)
            guard let addressName, addressName.name.isNotEmpty, addressName.name != field.value else {
                return address
            }
            return "\(addressName.name) (\(address))"
        case .timestamp:
            return relativeDateFormatter.string(fromTimestampValue: field.value)
        case .text:
            return field.value
        }
    }

    public var contextMenuItems: [ContextMenuItemType] {
        guard field.fieldType == .address else {
            return []
        }
        return [.copy(value: field.value)] + (explorerItem.map { [$0] } ?? [])
    }
}
