// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct AddressNameInfo: Codable, FetchableRecord {
    let addressName: AddressRecord
    let wallet: WalletRecord?
}

extension AddressNameInfo {
    func mapToAddressName() -> AddressName {
        AddressName(
            chain: addressName.chain,
            address: addressName.address,
            name: addressName.name,
            type: addressName.type,
            status: addressName.status,
            imageUrl: addressName.imageUrl ?? wallet?.imageUrl,
        )
    }
}
