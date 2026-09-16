// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import Style
import SwiftUI

public struct PerpetualDirectionViewModel {
    private let direction: PerpetualDirection

    public init(direction: PerpetualDirection) {
        self.direction = direction
    }

    public var title: String {
        direction.title
    }

    public var increaseTitle: String {
        Localized.Perpetual.increaseDirection(title)
    }

    public var reduceTitle: String {
        Localized.Perpetual.reduceDirection(title)
    }

    public var color: Color {
        direction.color
    }
}
