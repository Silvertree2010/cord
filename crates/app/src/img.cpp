#include "img.h"
#include "cord/src/img.cxxqt.h"

class MRes : public QQuickImageResponse, public QRunnable {
public:
    MRes(QString id, QSize rq) : m_id(std::move(id)), m_rq(rq) { setAutoDelete(false); }
    QQuickTextureFactory* textureFactory() const override { return QQuickTextureFactory::textureFactoryForImage(m_img); }
    void run() override {
        auto u = m_id.toStdString();
        auto v = media_get(rust::Str(u.data(), u.size()));
        m_img.loadFromData(reinterpret_cast<const uchar*>(v.data()), int(v.size()));
        if (m_rq.isValid() && !m_img.isNull()) m_img = m_img.scaled(m_rq, Qt::KeepAspectRatio, Qt::SmoothTransformation);
        emit finished();
    }
private:
    QString m_id;
    QSize m_rq;
    QImage m_img;
};

class MProv : public QQuickAsyncImageProvider {
public:
    QQuickImageResponse* requestImageResponse(const QString& id, const QSize& rq) override {
        auto* r = new MRes(id, rq);
        m_pool.start(r);
        return r;
    }
private:
    QThreadPool m_pool;
};

class VProv : public QQuickImageProvider {
public:
    VProv() : QQuickImageProvider(QQuickImageProvider::Image) {}
    QImage requestImage(const QString& id, QSize* sz, const QSize&) override {
        auto u = id.section('?', 0, 0).toStdString();
        uint32_t w = 0, h = 0;
        auto v = frame_get(rust::Str(u.data(), u.size()), w, h);
        if (!w || !h || v.size() < size_t(w) * h * 4) {
            if (sz) *sz = QSize(16, 9);
            QImage b(16, 9, QImage::Format_RGBA8888);
            b.fill(Qt::black);
            return b;
        }
        QImage im(reinterpret_cast<const uchar*>(v.data()), int(w), int(h), QImage::Format_RGBA8888);
        if (sz) *sz = QSize(int(w), int(h));
        return im.copy();
    }
};

void reg_img(QQmlApplicationEngine& e) {
    e.addImageProvider(QStringLiteral("cordm"), new MProv);
    e.addImageProvider(QStringLiteral("cordv"), new VProv);
}
