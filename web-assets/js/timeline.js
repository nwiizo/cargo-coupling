// =====================================================
// History Timeline Module
// =====================================================

import { CONFIG, state, setHistoryData } from './state.js';

const SVG_NS = 'http://www.w3.org/2000/svg';
const PLAY_INTERVAL_MS = 1000;
const CHART_WIDTH = 320;
const CHART_HEIGHT = 150;
const CHART_PADDING = 20;

let activeIndex = 0;
let playTimer = null;
let revisionSelectedCallback = null;

export async function initTimeline(options = {}) {
    const chart = document.getElementById('timeline-chart');
    if (!chart) return;
    revisionSelectedCallback = options.onRevisionSelected || null;

    try {
        const history = await fetchHistoryData();
        setHistoryData(history);
        renderTimeline(history);
        setupTimelineControls();
    } catch (error) {
        console.error('Failed to load history:', error);
        chart.innerHTML = '<p class="placeholder">History unavailable</p>';
    }
}

async function fetchHistoryData() {
    const url = CONFIG.apiEndpoint + CONFIG.historyPath;
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`HTTP ${response.status} from ${url}`);
    }
    return response.json();
}

function renderTimeline(history) {
    const chart = document.getElementById('timeline-chart');
    const scrubber = document.getElementById('timeline-scrubber');
    const playButton = document.getElementById('timeline-play');
    const points = history?.points || [];

    stopPlayback();

    if (!chart || !scrubber || !playButton) return;

    if (points.length === 0) {
        chart.innerHTML = '<p class="placeholder">No git history in this window</p>';
        scrubber.disabled = true;
        playButton.disabled = true;
        updateStats(null);
        return;
    }

    chart.textContent = '';
    chart.appendChild(buildChart(points));

    activeIndex = points.length - 1;
    scrubber.min = '0';
    scrubber.max = String(points.length - 1);
    scrubber.value = String(activeIndex);
    scrubber.disabled = points.length <= 1;
    playButton.disabled = points.length <= 1;

    setActiveIndex(activeIndex);
}

function buildChart(points) {
    const svg = document.createElementNS(SVG_NS, 'svg');
    svg.setAttribute('viewBox', `0 0 ${CHART_WIDTH} ${CHART_HEIGHT}`);
    svg.setAttribute('role', 'img');
    svg.setAttribute('aria-label', 'Coupling health history');
    svg.classList.add('timeline-svg');

    appendGrid(svg);

    const linePath = document.createElementNS(SVG_NS, 'path');
    linePath.setAttribute('class', 'timeline-line');
    linePath.setAttribute('d', pathForPoints(points));
    svg.appendChild(linePath);

    const playhead = document.createElementNS(SVG_NS, 'line');
    playhead.setAttribute('id', 'timeline-playhead');
    playhead.setAttribute('class', 'timeline-playhead');
    playhead.setAttribute('y1', String(CHART_PADDING));
    playhead.setAttribute('y2', String(CHART_HEIGHT - CHART_PADDING));
    svg.appendChild(playhead);

    points.forEach((point, index) => {
        const circle = document.createElementNS(SVG_NS, 'circle');
        circle.setAttribute('class', `timeline-point grade-${point.grade}`);
        circle.setAttribute('data-index', String(index));
        circle.setAttribute('cx', String(xForIndex(index, points.length)));
        circle.setAttribute('cy', String(yForScore(point.average_score)));
        circle.setAttribute('r', '4');
        circle.setAttribute('tabindex', '0');

        const title = document.createElementNS(SVG_NS, 'title');
        title.textContent = `${point.date} ${point.commit}: ${formatScore(point.average_score)} (${point.grade})`;
        circle.appendChild(title);

        circle.addEventListener('click', () => setActiveIndex(index, { loadGraph: true }));
        circle.addEventListener('keydown', (event) => {
            if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
                setActiveIndex(index, { loadGraph: true });
            }
        });

        svg.appendChild(circle);
    });

    return svg;
}

function appendGrid(svg) {
    [0.25, 0.5, 0.75, 1].forEach((score) => {
        const y = yForScore(score);
        const line = document.createElementNS(SVG_NS, 'line');
        line.setAttribute('class', 'timeline-grid-line');
        line.setAttribute('x1', String(CHART_PADDING));
        line.setAttribute('x2', String(CHART_WIDTH - CHART_PADDING));
        line.setAttribute('y1', String(y));
        line.setAttribute('y2', String(y));
        svg.appendChild(line);
    });
}

function setupTimelineControls() {
    const scrubber = document.getElementById('timeline-scrubber');
    const playButton = document.getElementById('timeline-play');

    scrubber?.addEventListener('input', (event) => {
        setActiveIndex(Number(event.target.value), { loadGraph: true });
    });

    playButton?.addEventListener('click', () => {
        if (playTimer) {
            stopPlayback();
        } else {
            startPlayback();
        }
    });
}

function startPlayback() {
    const points = state.historyData?.points || [];
    if (points.length <= 1) return;

    const playButton = document.getElementById('timeline-play');
    if (playButton) {
        playButton.textContent = 'Pause';
        playButton.classList.add('active');
    }

    playTimer = window.setInterval(() => {
        const next = activeIndex + 1 >= points.length ? 0 : activeIndex + 1;
        setActiveIndex(next, { loadGraph: true });
    }, PLAY_INTERVAL_MS);
}

function stopPlayback() {
    if (playTimer) {
        window.clearInterval(playTimer);
        playTimer = null;
    }

    const playButton = document.getElementById('timeline-play');
    if (playButton) {
        playButton.textContent = 'Play';
        playButton.classList.remove('active');
    }
}

function setActiveIndex(index, options = {}) {
    const points = state.historyData?.points || [];
    if (points.length === 0) return;

    activeIndex = Math.max(0, Math.min(index, points.length - 1));

    const scrubber = document.getElementById('timeline-scrubber');
    if (scrubber) {
        scrubber.value = String(activeIndex);
    }

    document.querySelectorAll('.timeline-point').forEach((point) => {
        const isActive = Number(point.dataset.index) === activeIndex;
        point.classList.toggle('active', isActive);
        point.setAttribute('r', isActive ? '6' : '4');
    });

    const x = xForIndex(activeIndex, points.length);
    const playhead = document.getElementById('timeline-playhead');
    if (playhead) {
        playhead.setAttribute('x1', String(x));
        playhead.setAttribute('x2', String(x));
    }

    updateStats(points[activeIndex]);
    if (options.loadGraph) {
        revisionSelectedCallback?.(points[activeIndex], activeIndex);
    }
}

function updateStats(point) {
    setText('timeline-date', point?.date || '-');
    setText('timeline-commit', point?.commit || '-');
    setText('timeline-score', point ? formatScore(point.average_score) : '-');
    setText('timeline-couplings', point ? String(point.total_couplings) : '-');
    setText('timeline-critical', point ? String(point.critical_issues) : '-');

    const grade = document.getElementById('timeline-grade');
    if (grade) {
        grade.textContent = point?.grade || '-';
        grade.className = point ? `health-grade ${point.grade}` : 'health-grade';
    }
}

function setText(id, value) {
    const element = document.getElementById(id);
    if (element) {
        element.textContent = value;
    }
}

function pathForPoints(points) {
    return points
        .map((point, index) => {
            const command = index === 0 ? 'M' : 'L';
            return `${command} ${xForIndex(index, points.length)} ${yForScore(point.average_score)}`;
        })
        .join(' ');
}

function xForIndex(index, total) {
    if (total <= 1) {
        return CHART_WIDTH / 2;
    }
    const usableWidth = CHART_WIDTH - CHART_PADDING * 2;
    return CHART_PADDING + (usableWidth * index) / (total - 1);
}

function yForScore(score) {
    const clamped = Math.max(0, Math.min(1, Number(score) || 0));
    const usableHeight = CHART_HEIGHT - CHART_PADDING * 2;
    return CHART_HEIGHT - CHART_PADDING - clamped * usableHeight;
}

function formatScore(score) {
    return Number(score).toFixed(3);
}
