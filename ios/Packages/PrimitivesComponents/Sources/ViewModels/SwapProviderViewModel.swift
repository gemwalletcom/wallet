// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.SwapProviderData
import GemstonePrimitives
import Primitives

public struct SwapProviderViewModel {
    private let providerData: Gemstone.SwapProviderData

    public init(providerData: Gemstone.SwapProviderData) {
        self.providerData = providerData
    }

    public var providerText: String {
        providerData.name
    }

    public var providerImage: AssetImage {
        AssetImage(
            imageURL: .none,
            placeholder: providerData.provider.map().image,
            chainPlaceholder: .none,
        )
    }
}
