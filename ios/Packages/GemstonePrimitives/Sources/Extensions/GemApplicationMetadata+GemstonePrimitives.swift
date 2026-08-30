// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemApplicationMetadataService
import Primitives

private let applicationMetadataService = GemApplicationMetadataService()

public extension Primitives.ApplicationMetadata {
    var shortName: String {
        return applicationMetadataService.shortName(metadata: json())
    }
}
