// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemApplicationMetadataService
import Primitives

public extension Primitives.ApplicationMetadata {
    var shortName: String {
        GemApplicationMetadataService().shortName(metadata: json())
    }
}
