// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

private struct ChartGesturesView: UIViewRepresentable {
    private enum Constants {
        static let scrubHold: TimeInterval = 0.1
    }

    @Binding var isPinching: Bool
    let onScrub: @MainActor (CGPoint) -> Void
    let onScrubEnd: @MainActor () -> Void
    let onZoom: @MainActor (Double) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(view: self)
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView()
        let scrub = UILongPressGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.onScrub))
        scrub.minimumPressDuration = Constants.scrubHold
        scrub.delegate = context.coordinator
        let pinch = UIPinchGestureRecognizer(target: context.coordinator, action: #selector(Coordinator.onPinch))
        pinch.delegate = context.coordinator
        view.addGestureRecognizer(scrub)
        view.addGestureRecognizer(pinch)
        return view
    }

    func updateUIView(_: UIView, context: Context) {
        context.coordinator.view = self
    }

    static func dismantleUIView(_: UIView, coordinator: Coordinator) {
        coordinator.view.isPinching = false
    }

    final class Coordinator: NSObject, UIGestureRecognizerDelegate {
        var view: ChartGesturesView

        init(view: ChartGesturesView) {
            self.view = view
        }

        @objc func onScrub(_ recognizer: UILongPressGestureRecognizer) {
            switch recognizer.state {
            case .began, .changed:
                if !view.isPinching {
                    view.onScrub(recognizer.location(in: recognizer.view))
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
                view.isPinching = true
                view.onScrubEnd()
            case .changed:
                view.onZoom(recognizer.scale)
                recognizer.scale = 1
            case .ended, .cancelled, .failed:
                view.isPinching = false
            default:
                break
            }
        }

        func gestureRecognizer(_: UIGestureRecognizer, shouldRecognizeSimultaneouslyWith _: UIGestureRecognizer) -> Bool {
            true
        }

        func gestureRecognizer(_: UIGestureRecognizer, shouldBeRequiredToFailBy other: UIGestureRecognizer) -> Bool {
            other is UIScreenEdgePanGestureRecognizer
        }
    }
}

// MARK: - View Modifier

public extension View {
    func chartGestures(
        isPinching: Binding<Bool>,
        onScrub: @escaping @MainActor (CGPoint) -> Void,
        onScrubEnd: @escaping @MainActor () -> Void,
        onZoom: @escaping @MainActor (Double) -> Void,
    ) -> some View {
        overlay {
            ChartGesturesView(isPinching: isPinching, onScrub: onScrub, onScrubEnd: onScrubEnd, onZoom: onZoom)
        }
    }
}
