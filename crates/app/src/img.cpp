#include "img.h"
#include "cord/src/img.cxxqt.h"
#include <QMetaObject>
#include <QPointer>
#include <QQuickWindow>
#include <QTimer>
#include <QElapsedTimer>
#include <QGuiApplication>
#include <QDir>
#include <atomic>
#include <memory>

class MRes : public QQuickImageResponse {
public:
    explicit MRes(std::shared_ptr<std::atomic<bool>> stop) : m_stop(std::move(stop)) {}
    QQuickTextureFactory* textureFactory() const override { return QQuickTextureFactory::textureFactoryForImage(m_img); }
    QString errorString() const override { return m_err; }
    void cancel() override { m_stop->store(true); }
    void done(const QImage& im, const QString& err) {
        m_img = im;
        m_err = err;
        emit finished();
    }
private:
    std::shared_ptr<std::atomic<bool>> m_stop;
    QImage m_img;
    QString m_err;
};

class MTask : public QRunnable {
public:
    MTask(QPointer<MRes> r, QObject* ctx, std::shared_ptr<std::atomic<bool>> stop, QString id, QSize rq)
        : m_r(r), m_ctx(ctx), m_stop(std::move(stop)), m_id(std::move(id)), m_rq(rq) {}
    void run() override {
        QImage img;
        QString err;
        if (!m_stop->load()) {
            auto u = m_id.toStdString();
            auto v = media_get(rust::Str(u.data(), u.size()));
            if (!img.loadFromData(reinterpret_cast<const uchar*>(v.data()), int(v.size()))) err = QStringLiteral("media load failed");
            else if (m_rq.isValid() && !img.isNull() && (img.width() > m_rq.width() || img.height() > m_rq.height())) img = img.scaled(m_rq, Qt::KeepAspectRatio, Qt::SmoothTransformation);
        } else {
            err = QStringLiteral("cancelled");
        }
        QPointer<MRes> r = m_r;
        QMetaObject::invokeMethod(m_ctx, [r, img, err]() { if (r) r->done(img, err); }, Qt::QueuedConnection);
    }
private:
    QPointer<MRes> m_r;
    QObject* m_ctx;
    std::shared_ptr<std::atomic<bool>> m_stop;
    QString m_id;
    QSize m_rq;
};

class MProv : public QQuickAsyncImageProvider {
public:
    MProv() { m_pool.setMaxThreadCount(12); }
    QQuickImageResponse* requestImageResponse(const QString& id, const QSize& rq) override {
        auto stop = std::make_shared<std::atomic<bool>>(false);
        auto* r = new MRes(stop);
        m_pool.start(new MTask(QPointer<MRes>(r), &m_ctx, stop, id, rq));
        return r;
    }
private:
    QObject m_ctx;
    QThreadPool m_pool;
};

static void fr_free(void* p) { delete static_cast<rust::Box<FrameRef>*>(p); }

class VProv : public QQuickImageProvider {
public:
    VProv() : QQuickImageProvider(QQuickImageProvider::Image) {}
    QImage requestImage(const QString& id, QSize* sz, const QSize&) override {
        auto u = id.section('?', 0, 0).toStdString();
        auto* b = new rust::Box<FrameRef>(frame_ref(rust::Str(u.data(), u.size())));
        uint32_t w = (*b)->fr_w(), h = (*b)->fr_h();
        auto d = (*b)->fr_data();
        if (!w || !h || d.size() < size_t(w) * h * 4) {
            delete b;
            if (sz) *sz = QSize(16, 9);
            QImage bl(16, 9, QImage::Format_RGBA8888);
            bl.fill(Qt::black);
            return bl;
        }
        if (sz) *sz = QSize(int(w), int(h));
        return QImage(d.data(), int(w), int(h), QImage::Format_RGBA8888, fr_free, b);
    }
};

void reg_img(QQmlApplicationEngine& e) {
    e.addImageProvider(QStringLiteral("cordm"), new MProv);
    e.addImageProvider(QStringLiteral("cordv"), new VProv);
    const QString shot = qEnvironmentVariable("CORD_SHOT");
    if (!shot.isEmpty()) {
        const int at = qEnvironmentVariableIntValue("CORD_SHOT_AT") > 0 ? qEnvironmentVariableIntValue("CORD_SHOT_AT") : 2500;
        QObject::connect(&e, &QQmlApplicationEngine::objectCreated, &e, [shot, at](QObject* o, const QUrl&) {
            auto* w = qobject_cast<QQuickWindow*>(o);
            if (!w) return;
            QTimer::singleShot(at, w, [w, shot]() {
                w->grabWindow().save(shot);
                QGuiApplication::quit();
            });
        });
    }
    const QString dir = qEnvironmentVariable("CORD_FILM");
    if (dir.isEmpty()) return;
    const int len = qEnvironmentVariableIntValue("CORD_FILM_LEN") > 0 ? qEnvironmentVariableIntValue("CORD_FILM_LEN") : 3000;
    QDir().mkpath(dir);
    QObject::connect(&e, &QQmlApplicationEngine::objectCreated, &e, [dir, len](QObject* o, const QUrl&) {
        auto* w = qobject_cast<QQuickWindow*>(o);
        if (!w) return;
        auto* t = new QTimer(w);
        auto* et = new QElapsedTimer;
        et->start();
        auto n = std::make_shared<int>(0);
        t->setInterval(16);
        QObject::connect(t, &QTimer::timeout, w, [w, t, et, n, dir, len]() {
            const qint64 ms = et->elapsed();
            QImage im = w->grabWindow().scaled(720, 450, Qt::IgnoreAspectRatio, Qt::SmoothTransformation);
            im.save(QStringLiteral("%1/%2_%3.png").arg(dir).arg(*n, 4, 10, QLatin1Char('0')).arg(ms));
            ++*n;
            if (ms > len) { t->stop(); QGuiApplication::quit(); }
        });
        t->start();
    });
}
