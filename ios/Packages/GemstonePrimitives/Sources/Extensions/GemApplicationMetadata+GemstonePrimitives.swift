// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemApplicationMetadataService
import Primitives

public extension Primitives.ApplicationMetadata {
    func shortName(applicationMetadataService: GemApplicationMetadataService) -> String {
        applicationMetadataService.shortName(metadata: json())
    }
}
