// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct ContactRecordInfo: FetchableRecord, Codable {
    var contact: ContactRecord
    var addresses: [ContactAddressRecord]
}

extension ContactRecordInfo {
    var contactData: ContactData {
        ContactData(
            contact: contact.toContact(),
            addresses: addresses.map { $0.toContactAddress() },
        )
    }
}
