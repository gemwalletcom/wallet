// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemApplicationMetadataService
import Primitives

private let applicationMetadataService = GemApplicationMetadataService()

public extension Primitives.ApplicationMetadata {
    var shortName: String {
        guard let metadata = try? json() else { return name }
        return applicationMetadataService.shortName(metadata: metadata)
    }
}
