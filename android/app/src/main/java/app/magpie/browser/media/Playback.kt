package app.magpie.browser.media

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.graphics.drawable.Icon
import android.media.MediaMetadata
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.IBinder
import androidx.compose.runtime.snapshotFlow
import androidx.core.app.ServiceCompat
import app.magpie.browser.Magpie
import app.magpie.browser.MainActivity
import app.magpie.browser.R
import app.magpie.browser.browser.Tab
import app.magpie.browser.theme.host
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch

private const val CHANNEL = "playback"
private const val CHANNEL_NAME = "Background play"
private const val NOTIFICATION_ID = 1
private const val SESSION_TAG = "Magpie"
private const val ACTION_PLAY = "app.magpie.browser.media.PLAY"
private const val ACTION_PAUSE = "app.magpie.browser.media.PAUSE"
private const val CONTROLS = PlaybackState.ACTION_PLAY or PlaybackState.ACTION_PAUSE or
    PlaybackState.ACTION_PLAY_PAUSE or PlaybackState.ACTION_STOP
private const val UNKNOWN_POSITION = PlaybackState.PLAYBACK_POSITION_UNKNOWN

class Playback : Service() {
    private val browser get() = (application as Magpie).browser
    private val scope = MainScope()
    private lateinit var session: MediaSession

    private val media: Media? get() = browser.playing?.media

    override fun onCreate() {
        super.onCreate()
        getSystemService(NotificationManager::class.java)
            .createNotificationChannel(NotificationChannel(CHANNEL, CHANNEL_NAME, NotificationManager.IMPORTANCE_LOW))
        session = MediaSession(this, SESSION_TAG).apply {
            setCallback(Controls())
            isActive = true
        }
        scope.launch(Dispatchers.Main.immediate) {
            snapshotFlow { browser.playing?.takeIf { browser.settings.backgroundPlay } }.collect(::show)
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_PLAY -> media?.controller?.play()
            ACTION_PAUSE -> media?.controller?.pause()
        }
        show(browser.playing?.takeIf { browser.settings.backgroundPlay })
        return START_NOT_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        scope.cancel()
        session.release()
        super.onDestroy()
    }

    private fun show(tab: Tab?) {
        tab?.media?.let { describe(tab, it) }
        ServiceCompat.startForeground(this, NOTIFICATION_ID, notification(tab), ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK)
        if (tab != null) return
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    private fun describe(tab: Tab, media: Media) {
        val state = if (media.playing) PlaybackState.STATE_PLAYING else PlaybackState.STATE_PAUSED
        session.setPlaybackState(PlaybackState.Builder().setActions(CONTROLS).setState(state, UNKNOWN_POSITION, 1f).build())
        session.setMetadata(
            MediaMetadata.Builder()
                .putString(MediaMetadata.METADATA_KEY_TITLE, title(tab, media))
                .putString(MediaMetadata.METADATA_KEY_ARTIST, artist(tab, media))
                .build(),
        )
    }

    private fun title(tab: Tab, media: Media): String = media.title.ifBlank { tab.label }

    private fun artist(tab: Tab, media: Media): String = media.artist.ifBlank { host(tab.url) }

    private fun command(action: String): PendingIntent =
        PendingIntent.getService(this, action.hashCode(), Intent(this, Playback::class.java).setAction(action), PendingIntent.FLAG_IMMUTABLE)

    private fun toggle(media: Media): Notification.Action {
        val (icon, label, action) = if (media.playing) {
            Triple(android.R.drawable.ic_media_pause, "Pause", ACTION_PAUSE)
        } else {
            Triple(android.R.drawable.ic_media_play, "Play", ACTION_PLAY)
        }
        return Notification.Action.Builder(Icon.createWithResource(this, icon), label, command(action)).build()
    }

    private fun notification(tab: Tab?): Notification {
        val openApp = PendingIntent.getActivity(this, 0, Intent(this, MainActivity::class.java), PendingIntent.FLAG_IMMUTABLE)
        val builder = Notification.Builder(this, CHANNEL)
            .setSmallIcon(R.drawable.ic_playback)
            .setContentIntent(openApp)
            .setVisibility(Notification.VISIBILITY_PUBLIC)
            .setStyle(Notification.MediaStyle().setMediaSession(session.sessionToken).setShowActionsInCompactView(0))
        val media = tab?.media ?: return builder.setContentTitle(SESSION_TAG).build()
        return builder
            .setContentTitle(title(tab, media))
            .setContentText(artist(tab, media))
            .addAction(toggle(media))
            .build()
    }

    private inner class Controls : MediaSession.Callback() {
        override fun onPlay() {
            media?.controller?.play()
        }

        override fun onPause() {
            media?.controller?.pause()
        }

        override fun onStop() {
            media?.controller?.stop()
        }
    }

    companion object {
        fun start(context: Context) {
            try {
                context.startForegroundService(Intent(context, Playback::class.java))
            } catch (backgrounded: IllegalStateException) {
                return
            }
        }
    }
}
