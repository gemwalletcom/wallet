// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Primitives.ApplicationMetadata {
    var shortName: String {
        guard let metadata = try? json() else { return name }
        return Gemstone.applicationMetadataShortName(metadata: metadata)
    }
}
