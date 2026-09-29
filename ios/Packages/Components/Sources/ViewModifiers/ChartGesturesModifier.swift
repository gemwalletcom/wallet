// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

private struct ChartGesturesView: UIViewRepresentable {
    private enum Constants {
        static let scrubHold: TimeInterval = 0.25
        static let glideStopVelocity: CGFloat = 20
    }

    let plot: CGRect
    @Binding var isPinching: Bool
    let onScrub: @MainActor (Double) -> Void
    let onScrubEnd: @MainActor () -> Void
    let onZoom: @MainActor (Double, Double) -> Void
    let onPan: @MainActor (Double) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(view: self)
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView()
        let coordinator = context.coordinator
        let scrub = UILongPressGestureRecognizer(target: coordinator, action: #selector(Coordinator.onScrub))
        scrub.minimumPressDuration = Constants.scrubHold
        let pinch = UIPinchGestureRecognizer(target: coordinator, action: #selector(Coordinator.onPinch))
        let pan = UIPanGestureRecognizer(target: coordinator, action: #selector(Coordinator.onPan))
        pan.maximumNumberOfTouches = 1
        for recognizer in [scrub, pinch, pan] {
            recognizer.delegate = coordinator
            view.addGestureRecognizer(recognizer)
        }
        coordinator.scrub = scrub
        coordinator.pan = pan
        return view
    }

    func updateUIView(_: UIView, context: Context) {
        context.coordinator.view = self
    }

    static func dismantleUIView(_: UIView, coordinator: Coordinator) {
        coordinator.stopGlide()
        if coordinator.view.isPinching {
            coordinator.view.isPinching = false
        }
    }

    final class Coordinator: NSObject, UIGestureRecognizerDelegate {
        var view: ChartGesturesView
        weak var scrub: UILongPressGestureRecognizer?
        weak var pan: UIPanGestureRecognizer?

        private var glide: CADisplayLink?
        private var glideVelocity: CGFloat = 0

        init(view: ChartGesturesView) {
            self.view = view
        }

        @objc func onScrub(_ recognizer: UILongPressGestureRecognizer) {
            switch recognizer.state {
            case .began, .changed:
                stopGlide()
                if !view.isPinching {
                    view.onScrub(fraction(at: recognizer.location(in: recognizer.view)))
                }
            case .ended, .cancelled, .failed:
                view.onScrubEnd()
            default:
                break
            }
        }

        @objc func onPinch(_ recognizer: UIPinchGestureRecognizer) {
            switch recognizer.state {
            case .began:
                stopGlide()
                pan?.isEnabled = false
                pan?.isEnabled = true
                view.isPinching = true
                view.onScrubEnd()
            case .changed:
                view.onZoom(recognizer.scale, fraction(at: recognizer.location(in: recognizer.view)))
                recognizer.scale = 1
            case .ended, .cancelled, .failed:
                view.isPinching = false
            default:
                break
            }
        }

        @objc func onPan(_ recognizer: UIPanGestureRecognizer) {
            switch recognizer.state {
            case .began:
                stopGlide()
            case .changed:
                view.onPan(fraction(of: recognizer.translation(in: recognizer.view).x))
                recognizer.setTranslation(.zero, in: recognizer.view)
            case .ended:
                startGlide(recognizer.velocity(in: recognizer.view).x)
            default:
                break
            }
        }

        func stopGlide() {
            glide?.invalidate()
            glide = nil
        }

        private func startGlide(_ velocity: CGFloat) {
            guard abs(velocity) > Constants.glideStopVelocity else { return }
            glideVelocity = velocity
            let link = CADisplayLink(target: self, selector: #selector(onGlide))
            link.add(to: .main, forMode: .common)
            glide = link
        }

        @objc private func onGlide(_ link: CADisplayLink) {
            let seconds = link.targetTimestamp - link.timestamp
            view.onPan(fraction(of: glideVelocity * seconds))
            glideVelocity *= pow(UIScrollView.DecelerationRate.fast.rawValue, seconds * 1000)
            if abs(glideVelocity) < Constants.glideStopVelocity {
                stopGlide()
            }
        }

        private func fraction(at location: CGPoint) -> Double {
            fraction(of: location.x - view.plot.minX)
        }

        private func fraction(of distance: CGFloat) -> Double {
            distance / view.plot.width
        }

        func gestureRecognizerShouldBegin(_ recognizer: UIGestureRecognizer) -> Bool {
            guard let pan = recognizer as? UIPanGestureRecognizer else { return true }
            let velocity = pan.velocity(in: pan.view)
            let isScrubbing = scrub.map { [.began, .changed].contains($0.state) } ?? false
            return !isScrubbing && abs(velocity.x) > abs(velocity.y)
        }

        func gestureRecognizer(_: UIGestureRecognizer, shouldRecognizeSimultaneouslyWith _: UIGestureRecognizer) -> Bool {
            true
        }

        func gestureRecognizer(_ recognizer: UIGestureRecognizer, shouldReceive event: UIEvent) -> Bool {
            if recognizer is UIPinchGestureRecognizer, let view = recognizer.view, let touches = event.touches(for: view), touches.count > 1, touches.contains(where: { $0.phase == .began }) {
                stopPageScroll(above: view)
            }
            return true
        }

        private func stopPageScroll(above view: UIView) {
            guard let parent = view.superview, let page = sequence(first: parent, next: { $0.superview }).lazy.compactMap({ $0 as? UIScrollView }).first else { return }
            page.panGestureRecognizer.isEnabled = false
            page.panGestureRecognizer.isEnabled = true
        }

        func gestureRecognizer(_ recognizer: UIGestureRecognizer, shouldBeRequiredToFailBy other: UIGestureRecognizer) -> Bool {
            isBackSwipe(other, above: recognizer.view) || (!(recognizer is UIPinchGestureRecognizer) && other === (other.view as? UIScrollView)?.panGestureRecognizer)
        }

        private func isBackSwipe(_ other: UIGestureRecognizer, above view: UIView?) -> Bool {
            guard !(other is UIScreenEdgePanGestureRecognizer) else { return true }
            guard #available(iOS 26, *), let view else { return false }
            let navigation = sequence(first: view as UIResponder, next: { $0.next }).lazy.compactMap { $0 as? UINavigationController }.first
            return other === navigation?.interactiveContentPopGestureRecognizer
        }
    }
}

// MARK: - View Modifier

public extension View {
    func chartGestures(
        in plot: CGRect,
        isPinching: Binding<Bool>,
        onScrub: @escaping @MainActor (Double) -> Void,
        onScrubEnd: @escaping @MainActor () -> Void,
        onZoom: @escaping @MainActor (Double, Double) -> Void,
        onPan: @escaping @MainActor (Double) -> Void,
    ) -> some View {
        overlay {
            ChartGesturesView(plot: plot, isPinching: isPinching, onScrub: onScrub, onScrubEnd: onScrubEnd, onZoom: onZoom, onPan: onPan)
        }
    }
}
