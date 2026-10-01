import Foundation
import SwiftUI

public typealias Spacing = CGFloat
public typealias Sizing = CGFloat

public extension Spacing {
    static let space1: Spacing = 1
    static let space2: Spacing = 2
    static let space4: Spacing = 4
    static let space6: Spacing = 6
    static let space8: Spacing = 8
    static let space10: Spacing = 10
    static let space12: Spacing = 12
    static let space16: Spacing = 16
    static let space24: Spacing = 24
    static let space32: Spacing = 32

    static let extraSmall: CGFloat = space2
    static let tiny: CGFloat = space4
    static let small: CGFloat = space8
    static let medium: CGFloat = space16
    static let large: CGFloat = space24
    static let extraLarge: CGFloat = space32

    enum scene {
        public static let top: CGFloat = space16
        public static let bottom: CGFloat = space8

        public enum button {
            public static let maxWidth: CGFloat = 340
            public static let height: CGFloat = 50
        }

        public enum content {
            public static let maxWidth: CGFloat = 360
        }
    }
}

public extension Sizing {
    enum button {
        public static let paddingHorizontal: CGFloat = .space12
        public static let paddingVertical: CGFloat = .space12
    }

    enum image {
        public static let small: CGFloat = 22
        public static let semiMedium: CGFloat = 34
        public static let medium: CGFloat = 44
        public static let semiLarge: CGFloat = 64
        public static let large: CGFloat = 88
        public static let semiExtraLarge: CGFloat = 102
        public static let extraLarge: CGFloat = 120

        public static let asset: CGFloat = 44
        public static let app: CGFloat = Self.asset
    }

    enum list {
        public static let minHeight: CGFloat = 84
        public static let accessory: CGFloat = 16
        public static let image: CGFloat = 22
        public static let settings: CGFloat = 28

        public enum selected {
            public static let image: CGFloat = 20
        }

        public enum assets {
            public static let widget: CGFloat = 40
        }
    }

    enum shadow {
        public static let radius: CGFloat = 10
        public static let yOffset: CGFloat = 5
    }

    enum picker {
        public static let segmentedWidth: CGFloat = 200
    }

    enum chart {
        public static let height: CGFloat = 320
    }
}
