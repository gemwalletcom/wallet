// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

enum ImageState: Equatable {
    case empty
    case success(UIImage)
    case failure(Error)

    static func == (lhs: ImageState, rhs: ImageState) -> Bool {
        switch (lhs, rhs) {
        case (.empty, .empty):
            true
        case let (.success(lhsImage), .success(rhsImage)):
            lhsImage.hashValue == rhsImage.hashValue
        case let (.failure(lhsError), .failure(rhsError)):
            lhsError.localizedDescription == rhsError.localizedDescription
        default:
            false
        }
    }
}
