// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public extension Font {
    enum app {
        public static let extraLargeTitle: Font = .system(size: 72)

        public static let display: Font = .system(size: 44, weight: .semibold)

        public static let largeTitle: Font = .system(size: 42, weight: .semibold)

        public static let title1: Font = .system(size: 36, weight: .semibold)

        public static let title2: Font = .system(size: 22, weight: .medium)

        public static let title3: Font = .system(size: 20, weight: .medium)

        public static let headline: Font = .system(size: 17, weight: .medium)

        public static let body: Font = .system(size: 16, weight: .medium)

        public static let callout: Font = .system(size: 13, weight: .medium)

        public static let footnote: Font = .system(size: 12, weight: .medium)

        public static let caption: Font = .system(size: 10, weight: .semibold)

        public enum Widget {
            public static let title: Font = .system(size: 32, weight: .bold)

            public static let body: Font = .system(size: 16, weight: .regular)

            public static let headline: Font = .system(size: 16, weight: .semibold)

            public static let callout: Font = .system(size: 14, weight: .medium)
        }
    }
}
