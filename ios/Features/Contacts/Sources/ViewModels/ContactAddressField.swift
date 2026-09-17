// Copyright (c). Gem Wallet. All rights reserved.

enum ContactAddressField: String, Identifiable {
    case network
    case address
    case memo

    var id: String { rawValue }
}
