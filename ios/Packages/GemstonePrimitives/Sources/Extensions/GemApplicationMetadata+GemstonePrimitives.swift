// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemApplicationMetadataService
import Primitives

public extension Primitives.ApplicationMetadata {
    var iconURL: URL? {
        GemApplicationMetadataService.shared.iconUrl(metadata: map()).flatMap(URL.init(string:))
    }

    var shortName: String {
        GemApplicationMetadataService.shared.shortName(metadata: map())
    }
}
