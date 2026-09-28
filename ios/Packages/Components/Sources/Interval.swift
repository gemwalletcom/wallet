// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public typealias Interval = TimeInterval

public extension Interval {
    enum AnimationDuration {
        public static let fast: Interval = 0.15
        public static let normal: Interval = 0.2
        public static let slow: Interval = 0.5
        public static let verySlow: Interval = 1.8
    }
}

public extension Duration {
    static let debounce: Duration = .milliseconds(250)
}
