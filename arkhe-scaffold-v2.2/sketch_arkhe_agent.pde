/**
 * Arkhe Agent Visualizer — Processing 4
 *
 * Displays:
 *   - Learning curve (verification rate + veto rate over episodes)
 *   - Focal loss trend
 *   - Health score trend
 *   - Trust level distribution (bar chart)
 *   - Real-time veto event log
 *
 * For standalone demo, runs a built-in simulation.
 * No external dependencies required.
 */

import java.util.ArrayDeque;
import java.util.Queue;

// ── Simulation State ──
int MAX_EPISODES = 100;
int POINTS_VISIBLE = 50;

float[] verifyRate = new float[MAX_EPISODES];
float[] vetoRate = new float[MAX_EPISODES];
float[] focalLoss = new float[MAX_EPISODES];
float[] healthScore = new float[MAX_EPISODES];
int currentEpisode = 0;

// Trust level counters
int[] trustCounts = {0, 0, 0, 0, 0}; // L0..L4

// Veto event log
Queue<VetoEvent> vetoLog = new ArrayDeque<VetoEvent>();

// Layout constants
int MARGIN = 60;
int CURVE_H = 140;
int CURVE_GAP = 30;
int BAR_W = 36;
int BAR_GAP = 16;

// Colors
color COL_BG = #0D1117;
color COL_GRID = #21262D;
color COL_TEXT = #C9D1D9;
color COL_TEXT_DIM = #8B949E;
color COL_VERIFY = #3FB950;
color COL_VETO = #F85149;
color COL_FOCAL = #D29922;
color COL_HEALTH = #58A6FF;
color COL_ACCENT = #BC8CFF;
color COL_BORDER = #30363D;

// Trust level colors
color[] TRUST_COLORS = {
  #484F58, // L0 gray
  #D29922, // L1 yellow
  #3FB950, // L2 green
  #58A6FF, // L3 blue
  #BC8CFF, // L4 purple
};

String[] TRUST_LABELS = {"L0", "L1", "L2", "L3", "L4"};

// ── Veto Event ──
class VetoEvent {
  int episode;
  String reason;
  float score;
  VetoEvent(int ep, String r, float s) {
    episode = ep;
    reason = r;
    score = s;
  }
}

// ── Simulation ──
void simulateEpisode(int ep) {
  // Base verification rate improves over time (learning curve)
  float base = 0.5 + 0.35 * (1 - exp(-ep * 0.03));
  float noise = random(-0.04, 0.04);

  // Simulate overfitting spike around episodes 55-75
  float overfitSpike = 0;
  if (ep > 55 && ep < 75) {
    overfitSpike = -0.15 * sin(map(ep, 55, 75, 0, PI));
  }

  verifyRate[ep] = constrain(base + noise + overfitSpike, 0, 1);
  vetoRate[ep] = constrain(1.0 - verifyRate[ep] + random(-0.02, 0.02), 0, 1);
  focalLoss[ep] = constrain(0.5 * exp(-ep * 0.02) + random(0, 0.04), 0.01, 1);
  healthScore[ep] = constrain(
    verifyRate[ep] - 0.3 * vetoRate[ep] - 0.1 * focalLoss[ep], 0, 1
  );

  // Assign trust levels (improving distribution over time)
  float r = random(1);
  float l4Threshold = 0.05 + ep * 0.003; // L4 becomes more common
  if (r < 0.08) trustCounts[0]++;
  else if (r < 0.20) trustCounts[1]++;
  else if (r < 0.50) trustCounts[2]++;
  else if (r < l4Threshold + 0.60) trustCounts[3]++;
  else trustCounts[4]++;

  // Generate veto events probabilistically
  if (random(1) < vetoRate[ep] * 0.25) {
    String[] reasons = {"evidence_gap", "consistency_gap", "semantic_drift", "focal_ambiguous"};
    String reason = reasons[(int)random(reasons.length)];
    float score = random(0.3, 3.0);
    vetoLog.add(new VetoEvent(ep, reason, score));
    if (vetoLog.size() > 8) vetoLog.poll();
  }
}

// ── Drawing Helpers ──

void drawPanel(int x, int y, int w, int h, String title) {
  stroke(COL_BORDER);
  strokeWeight(1);
  noFill();
  rect(x, y, w, h, 4);

  noStroke();
  fill(COL_ACCENT);
  textSize(11);
  textAlign(LEFT, TOP);
  text(title, x + 8, y + 6);
}

void drawGrid(int x, int y, int w, int h, int rows, int cols) {
  stroke(COL_GRID);
  strokeWeight(1);
  for (int i = 0; i <= rows; i++) {
    float yy = y + (h * i / (float)rows);
    line(x, yy, x + w, yy);
  }
  for (int j = 0; j <= cols; j++) {
    float xx = x + (w * j / (float)cols);
    line(xx, y, xx, y + h);
  }

  // Y-axis labels
  fill(COL_TEXT_DIM);
  noStroke();
  textSize(9);
  textAlign(RIGHT, CENTER);
  for (int i = 0; i <= rows; i++) {
    float val = 1.0 - i / (float)rows;
    float yy = y + (h * i / (float)rows);
    text(nf(val, 1, 1), x - 6, yy);
  }
}

void drawCurve(float[] data, int count, int x, int y, int w, int h, color c, String label) {
  // Legend
  noStroke();
  fill(c);
  textSize(10);
  textAlign(LEFT, BOTTOM);
  text("● " + label, x + 4, y - 2);

  if (count < 2) return;

  // Curve
  stroke(c);
  strokeWeight(2);
  noFill();
  beginShape();
  int start = max(0, count - POINTS_VISIBLE);
  int visibleCount = min(count, POINTS_VISIBLE);
  for (int i = 0; i < visibleCount; i++) {
    int idx = start + i;
    float px = x + map(i, 0, visibleCount - 1, 0, w);
    float py = y + h - data[idx] * h;
    vertex(px, py);
  }
  endShape();

  // Current value dot + label
  if (count > 0) {
    float lastVal = data[count - 1];
    float lastX = x + w;
    float lastY = y + h - lastVal * h;

    fill(c);
    noStroke();
    ellipse(lastX, lastY, 7, 7);
    fill(COL_TEXT);
    textSize(10);
    textAlign(RIGHT, BOTTOM);
    text(nf(lastVal, 1, 3), lastX - 10, lastY - 5);
  }
}

void drawTrustBars(int x, int y, int w, int h) {
  int totalBars = 5;
  int totalWidth = totalBars * BAR_W + (totalBars - 1) * BAR_GAP;
  int offsetX = x + (w - totalWidth) / 2;

  // Title
  noStroke();
  fill(COL_TEXT);
  textSize(11);
  textAlign(CENTER, BOTTOM);
  text("Trust Level Distribution", x + w / 2, y - 6);

  int total = 0;
  for (int c : trustCounts) total += c;
  if (total == 0) return;

  int maxCount = 0;
  for (int c : trustCounts) if (c > maxCount) maxCount = c;
  if (maxCount == 0) maxCount = 1;

  for (int i = 0; i < 5; i++) {
    float barH = map(trustCounts[i], 0, maxCount, 0, h);
    int bx = offsetX + i * (BAR_W + BAR_GAP);

    // Bar
    fill(TRUST_COLORS[i]);
    noStroke();
    rect(bx, y + h - barH, BAR_W, barH, 3, 3, 0, 0);

    // Level label
    fill(COL_TEXT);
    textSize(10);
    textAlign(CENTER, TOP);
    text(TRUST_LABELS[i], bx + BAR_W / 2, y + h + 5);

    // Count
    textSize(9);
    fill(COL_TEXT_DIM);
    text(trustCounts[i], bx + BAR_W / 2, y + h + 19);

    // Percentage
    text(nf(trustCounts[i] / (float)total * 100, 1, 0) + "%", bx + BAR_W / 2, y + h + 31);
  }
}

void drawVetoLog(int x, int y, int w, int h) {
  // Title
  noStroke();
  fill(COL_TEXT);
  textSize(11);
  textAlign(LEFT, BOTTOM);
  text("Recent Veto Events", x, y - 4);

  if (vetoLog.isEmpty()) {
    fill(COL_TEXT_DIM);
    textSize(10);
    textAlign(LEFT, TOP);
    text("No veto events yet...", x + 4, y + 4);
    return;
  }

  textSize(10);
  int i = 0;
  for (VetoEvent ve : vetoLog) {
    float vy = y + 4 + i * 18;
    if (vy + 14 > y + h) break; // Don't overflow panel

    // Red dot
    fill(COL_VETO);
    noStroke();
    ellipse(x + 6, vy + 6, 6, 6);

    // Text
    fill(COL_TEXT);
    textAlign(LEFT, TOP);
    text(
      "Ep " + nf(ve.episode, 3) +
      "  score=" + nf(ve.score, 1, 2) +
      "  " + ve.reason,
      x + 16, vy
    );
    i++;
  }
}

// ── Main ──

void setup() {
  size(1000, 780);
  textFont("monospace");
  frameRate(30);
  noSmooth(); // Crisp lines for data viz
}

void draw() {
  background(COL_BG);

  // Simulate next episode (one per frame)
  if (currentEpisode < MAX_EPISODES) {
    simulateEpisode(currentEpisode);
    currentEpisode++;
  }

  int n = currentEpisode;
  int plotW = width - 2 * MARGIN - 40; // Leave room for Y-axis labels
  int plotX = MARGIN + 35;

  // ── Header ──
  noStroke();
  fill(COL_ACCENT);
  textSize(16);
  textAlign(LEFT, TOP);
  text("ARKHE AGENT MONITOR", MARGIN, 15);

  fill(COL_TEXT_DIM);
  textSize(11);
  textAlign(RIGHT, TOP);
  text("v20.2  |  Episode " + (n > 0 ? n - 1 : 0) + "/" + MAX_EPISODES, width - MARGIN, 17);

  // Status indicator
  String status = n >= MAX_EPISODES ? "COMPLETE" : "RUNNING";
  color statusCol = n >= MAX_EPISODES ? COL_VERIFY : COL_FOCAL;
  fill(statusCol);
  textAlign(RIGHT, TOP);
  textSize(10);
  text("● " + status, width - MARGIN, 32);

  // Divider
  stroke(COL_BORDER);
  strokeWeight(1);
  line(MARGIN, 48, width - MARGIN, 48);

  // ── Panel 1: Verification + Veto Rate ──
  int y1 = MARGIN + 10;
  drawPanel(MARGIN + 20, y1 - 16, plotW + 35, CURVE_H + 24, "");
  drawGrid(plotX, y1, plotW, CURVE_H, 4, 10);
  drawCurve(verifyRate, n, plotX, y1, plotW, CURVE_H, COL_VERIFY, "Verify Rate");
  drawCurve(vetoRate, n, plotX, y1, plotW, CURVE_H, COL_VETO, "Veto Rate");

  // ── Panel 2: Focal Loss ──
  int y2 = y1 + CURVE_H + CURVE_GAP;
  drawPanel(MARGIN + 20, y2 - 16, plotW + 35, CURVE_H + 24, "");
  drawGrid(plotX, y2, plotW, CURVE_H, 4, 10);
  drawCurve(focalLoss, n, plotX, y2, plotW, CURVE_H, COL_FOCAL, "Focal Loss");

  // ── Panel 3: Health Score ──
  int y3 = y2 + CURVE_H + CURVE_GAP;
  drawPanel(MARGIN + 20, y3 - 16, plotW + 35, CURVE_H + 24, "");
  drawGrid(plotX, y3, plotW, CURVE_H, 4, 10);
  drawCurve(healthScore, n, plotX, y3, plotW, CURVE_H, COL_HEALTH, "Health Score");

  // ── Bottom section: Trust bars + Veto log side by side ──
  int bottomY = y3 + CURVE_H + 30;
  int bottomH = height - bottomY - 20;
  int halfW = (plotW + 35) / 2 - 10;

  // Trust bars (left half)
  int barAreaH = bottomH - 50; // Leave room for labels below
  drawPanel(MARGIN + 20, bottomY - 16, halfW, bottomH + 16, "");
  drawTrustBars(MARGIN + 20, bottomY, halfW, barAreaH);

  // Veto log (right half)
  int logX = MARGIN + 20 + halfW + 20;
  int logW = halfW;
  drawPanel(logX, bottomY - 16, logW, bottomH + 16, "");
  drawVetoLog(logX + 8, bottomY, logW - 16, bottomH - 10);

  // ── Bottom-right: Summary stats ──
  if (n > 0) {
    int sx = logX + logW + 20;
    int sy = bottomY;
    // Only draw if there's space
    if (sx + 120 < width) {
      noStroke();
      fill(COL_TEXT);
      textSize(10);
      textAlign(LEFT, TOP);
      int last = n - 1;
      text("Latest:", sx, sy);
      fill(COL_VERIFY);
      text("VR: " + nf(verifyRate[last], 1, 3), sx, sy + 16);
      fill(COL_VETO);
      text("VtR: " + nf(vetoRate[last], 1, 3), sx, sy + 30);
      fill(COL_FOCAL);
      text("FL: " + nf(focalLoss[last], 1, 3), sx, sy + 44);
      fill(COL_HEALTH);
      text("HP: " + nf(healthScore[last], 1, 3), sx, sy + 58);
    }
  }
}

// Restart simulation on key press
void keyPressed() {
  if (key == 'r' || key == 'R') {
    currentEpisode = 0;
    verifyRate = new float[MAX_EPISODES];
    vetoRate = new float[MAX_EPISODES];
    focalLoss = new float[MAX_EPISODES];
    healthScore = new float[MAX_EPISODES];
    trustCounts = new int[]{0, 0, 0, 0, 0};
    vetoLog.clear();
  }
}
