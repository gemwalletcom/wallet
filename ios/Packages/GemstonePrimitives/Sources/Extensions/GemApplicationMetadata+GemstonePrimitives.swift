// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemApplicationMetadataService
import Primitives

public extension Primitives.ApplicationMetadata {
    var iconURL: URL? {
        GemApplicationMetadataService.shared.iconUrl(metadata: toGem()).flatMap(URL.init(string:))
    }

    var shortName: String {
        GemApplicationMetadataService.shared.shortName(metadata: toGem())
    }

    var host: String {
        GemApplicationMetadataService.shared.host(metadata: toGem())
    }
}
