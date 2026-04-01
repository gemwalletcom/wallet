// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public extension Images {
    static func name(_ name: String) -> Image {
        Image(name, bundle: BundleToken.bundle)
    }
}

public extension UIImage {
    static func name(_ name: String) -> UIImage? {
        UIImage(named: name, in: BundleToken.bundle, with: .none)
    }
}

private final class BundleToken {
    static let bundle: Bundle = {
        #if SWIFT_PACKAGE
            return Bundle.module
        #else
            return Bundle(for: BundleToken.self)
        #endif
    }()
}
